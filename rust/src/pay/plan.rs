use std::{collections::HashMap, convert::Infallible, str::FromStr as _, sync::LazyLock};

use anyhow::{anyhow, Result};

use bip32::PrivateKey;
use itertools::Itertools;
use orchard::{
    circuit::ProvingKey,
    keys::{Scope, SpendAuthorizingKey},
    note::AssetBase,
    value::NoteValue,
    Address,
};
use pczt::{
    roles::{
        creator::Creator, io_finalizer::IoFinalizer, issuer::Issuer, prover::Prover,
        signer::Signer, spend_finalizer::SpendFinalizer, tx_extractor::TransactionExtractor,
        updater::Updater,
    },
    Pczt,
};
use rand_core::{OsRng, RngCore};
use ripemd::Ripemd160;
use sapling_crypto::PaymentAddress;
use secp256k1::{PublicKey, SecretKey};
use sha2::{Digest as _, Sha256};
use sqlx::{sqlite::SqliteRow, Row, SqliteConnection};
use tracing::{event, info, span, Level};
use zcash_address::{unified::Receiver, ConversionError, TryFromAddress, ZcashAddress};
use zcash_keys::{address::UnifiedAddress, encoding::AddressCodec as _};
use zcash_protocol::{PoolType, ShieldedPool};
use zcash_primitives::transaction::{
    builder::{BuildConfig, Builder},
    fees::zip317::FeeRule,
};
use zcash_proofs::prover::LocalTxProver;
use zcash_protocol::{
    consensus::{BlockHeight, NetworkType, NetworkUpgrade, Parameters},
    memo::{Memo, MemoBytes},
    value::Zatoshis,
};
use zcash_transparent::{
    address::TransparentAddress,
    builder::{SpendInfo, TransparentInputInfo},
    bundle::{OutPoint, TxOut},
    pczt::Bip32Derivation,
};
use zip321::{Payment, TransactionRequest};

use crate::{
    account::{
        derive_transparent_sk, generate_next_change_address, get_account_full_address,
        get_orchard_note, get_orchard_sk, get_orchard_vk, get_sapling_note, get_sapling_sk,
        get_sapling_vk,
    },
    api::{coin::Network, issuance::IssuanceInfo, pay::PcztPackage},
    db::{get_account_dindex, get_account_hw, select_account_transparent},
    keys::{sapling_pgk_for_scope, sapling_ssk_for_scope, SaplingFullViewingKey},
    pay::{
        error::Error,
        fee::COST_PER_ACTION,
        pool::{PoolMask, NUM_POOLS},
        prepare::to_zec,
        solve, InputNote, Recipient, RecipientState, ReceiverOption, DecomposedRecipient,
    },
    warp::hasher::{empty_roots, OrchardHasher, SaplingHasher},
    Client,
};

use zcash_primitives::transaction::zsa_builder::ZsaBuilder;

pub fn is_tex(network: &Network, address: &str) -> Result<bool> {
    let zaddress = ZcashAddress::from_str(address)?;
    let zaddress: zcash_keys::address::Address =
        zaddress.convert_if_network(network.network_type()).unwrap();

    let is_tex = matches!(zaddress, zcash_keys::address::Address::Tex(_));
    Ok(is_tex)
}

pub async fn build_puri(recipients: &[Recipient]) -> Result<String> {
    // make a payment uri
    let payments = recipients
        .iter()
        .map(|r| {
            let address = ZcashAddress::from_str(&r.address)?;
            let amount = Zatoshis::const_from_u64(r.amount);
            let memo = encode_memo(r)?;
            Ok::<_, anyhow::Error>(
                Payment::new(address, Some(amount), memo, None, None, vec![]).expect("payment"),
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let puri = TransactionRequest::new(payments)?;
    let puri = puri.to_uri();

    Ok(puri)
}

fn build_zsa_builder(info: &IssuanceInfo, oaddress: orchard::Address) -> Result<ZsaBuilder> {
    let mut zsa = ZsaBuilder::new(info.isk.clone());
    zsa.add_issue_output(
        info.desc_hash,
        oaddress,
        NoteValue::from_raw(info.amount),
        info.first_issuance,
        &mut OsRng,
    )
    .map_err(|e| anyhow!("Failed to add issue output: {e:?}"))?;
    if info.finalize {
        zsa.finalize_asset(&info.desc_hash)
            .map_err(|e| anyhow!("Failed to finalize asset: {e:?}"))?;
    }
    Ok(zsa)
}

/// Decompose a Zcash address into its individual shielded receivers.
/// - UA address → S/O/I receivers (transparent stripped)
/// - Pre-ironwood: Ironwood removed → max S/O
/// - Post-ironwood: Orchard removed → max S/I
/// - Single-pool address → 1 receiver as-is
/// Returns 1 or 2 ReceiverOptions (OR alternatives).
/// Prefer O/I over S, returning exactly 1 shielded receiver.
fn decompose_address(
    address: &str,
    network: &Network,
    ironwood_active: bool,
) -> Result<ReceiverOption> {
    // Decode as unified address (works for UAs and single-pool shielded)
    if let Ok(ua) = UnifiedAddress::decode(network, address) {
        // Prefer Orchard/Ironwood over Sapling
        if let Some(orchard) = ua.orchard() {
            return Ok(ReceiverOption {
                receiver: Receiver::Orchard(orchard.to_raw_address_bytes()),
                pool: if ironwood_active { 3 } else { 2 },
                remaining: 0,
            });
        }
        if let Some(sapling) = ua.sapling() {
            return Ok(ReceiverOption {
                receiver: Receiver::Sapling(sapling.to_bytes()),
                pool: 1,
                remaining: 0,
            });
        }
        anyhow::bail!("Address has no shielded receivers");
    }

    // Fallback: single-pool address (transparent, sapling, orchard).
    // UnifiedAddress::decode only handles Bech32m UA containers, so
    // regtest Sapling (Bech32) and transparent (Base58) addresses
    // must be decoded individually via the AddressCodec trait.
    let zaddr = ZcashAddress::try_from_encoded(address)?;

    if zaddr.can_receive_as(PoolType::Transparent) {
        let taddr = TransparentAddress::decode(network, address)
            .map_err(|e| anyhow!("Failed to decode transparent address: {e:?}"))?;
        let receiver = match taddr {
            TransparentAddress::PublicKeyHash(hash) => Receiver::P2pkh(hash),
            TransparentAddress::ScriptHash(hash) => Receiver::P2sh(hash),
        };
        return Ok(ReceiverOption { receiver, pool: 0, remaining: 0 });
    }

    if zaddr.can_receive_as(PoolType::Shielded(ShieldedPool::Sapling)) {
        let sapling = PaymentAddress::decode(network, address)
            .map_err(|e| anyhow!("Failed to decode sapling address: {e}"))?;
        return Ok(ReceiverOption {
            receiver: Receiver::Sapling(sapling.to_bytes()),
            pool: 1,
            remaining: 0,
        });
    }

    if zaddr.can_receive_as(PoolType::Shielded(ShieldedPool::Orchard)) {
        // Orchard single-pool addresses — re-decode through UA
        let ua = UnifiedAddress::decode(network, address)
            .map_err(|e| anyhow!("Failed to decode orchard address: {e}"))?;
        let orchard = ua
            .orchard()
            .ok_or_else(|| anyhow!("Address has no orchard receiver"))?;
        return Ok(ReceiverOption {
            receiver: Receiver::Orchard(orchard.to_raw_address_bytes()),
            pool: if ironwood_active { 3 } else { 2 },
            remaining: 0,
        });
    }

    anyhow::bail!("Unrecognized address pool");
}

#[allow(clippy::too_many_arguments)]
pub async fn plan_transaction(
    network: &Network,
    connection: &mut SqliteConnection,
    client: &mut Client,
    account: u32,
    src_pools: u8,
    recipients: &[Recipient],
    recipient_pays_fee: bool,
    confirmations: Option<u32>,
    smart_transparent: bool,
    category: Option<u32>,
    issuance: Option<&IssuanceInfo>,
    migration: bool,
    mode: crate::pay::solve::Mode,
    preselected: Option<&[u32]>,
) -> Result<PcztPackage> {
    let mut input_pools = fetch_unspent_notes_by_pool(connection, account).await?;
    let height = client.latest_height().await?;
    let confirmations = confirmations.unwrap_or_default();
    let max_height = height.saturating_sub(confirmations);
    for pool in 0..NUM_POOLS {
        if src_pools & (1 << pool) == 0 {
            input_pools[pool].clear();
        } else {
            input_pools[pool].retain(|n| n.height <= max_height);
        }
    }

    // Preselected filter: restrict to specific note IDs (e.g. migration)
    if let Some(ids) = preselected {
        for pool in 0..NUM_POOLS {
            input_pools[pool].retain(|n| ids.contains(&n.id));
        }
    }

    let recipients = recipients.to_vec();
    let (mut input_pools, recipients, recipient_pays_fee) = if smart_transparent {
        let mut notes = std::mem::take(&mut input_pools[0]);
        // Group by taddress, pick one random address to shield
        notes.sort_by_key(|n| n.taddress);
        let groups: Vec<Vec<InputNote>> = notes
            .into_iter()
            .chunk_by(|n| n.taddress)
            .into_iter()
            .map(|(_, group)| group.collect())
            .collect();
        let notes = if groups.is_empty() {
            vec![]
        } else {
            let i = OsRng.next_u32() as usize % groups.len();
            groups[i].clone()
        };
        let max = notes.iter().map(|n| n.amount).sum::<u64>();
        let recipient = Recipient {
            amount: max,
            ..recipients.first().cloned().unwrap_or_default()
        };
        let mut pools = vec![vec![]; NUM_POOLS as usize];
        pools[0] = notes;
        (pools, vec![recipient], true)
    } else {
        (input_pools, recipients, recipient_pays_fee)
    };

    let ironwood_active = network
        .is_nu_active(NetworkUpgrade::Nu6_3, BlockHeight::from_u32(height));
    let decomposed: Vec<DecomposedRecipient> = recipients
        .iter()
        .map(|r| {
            let asset_base = if r.asset_base.is_empty() {
                [0u8; 32].to_vec()
            } else {
                r.asset_base.clone()
            };
            Ok(DecomposedRecipient {
                address: r.address.clone(),
                receiver: decompose_address(&r.address, network, ironwood_active)?,
                amount: r.amount,
                remaining: r.amount,
                memo: r.user_memo.clone(),
                memo_bytes: r.memo_bytes.clone(),
                asset_base,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    // ZSA and Ironwood are mutually exclusive (different V6 version group IDs).
    let has_zsa = decomposed.iter().any(|d| d.asset_base != [0u8; 32].to_vec())
        || issuance.is_some();
    if has_zsa && ironwood_active {
        anyhow::bail!("ZSA and Ironwood are incompatible");
    }

    // Build asset list for solver: index 0 = ZEC, indices 1+ = ZSA (sorted)
    let zec_key = [0u8; 32];
    let zsa_assets: Vec<[u8; 32]> = decomposed
        .iter()
        .filter(|d| d.asset_base != zec_key.to_vec())
        .map(|d| d.asset_base.clone())
        .sorted()
        .dedup()
        .filter_map(|b| b.try_into().ok())
        .collect();

    // ── Compute additional context ───────────────────────────────────────
    let dindex = get_account_dindex(connection, account).await?;
    let hw = get_account_hw(&mut *connection, account).await?;

    // Compute weighted average price from recipients that have a price set
    let mut total_amount = 0;
    let mut total_fiat = 0.0;
    for r in &recipients {
        if let Some(p) = r.price {
            total_fiat += p * r.amount as f64;
            total_amount += r.amount;
        }
    }
    let price = if total_amount != 0 {
        Some(total_fiat / total_amount as f64)
    } else {
        None
    };

    let (use_internal,): (bool,) =
        sqlx::query_as("SELECT use_internal FROM accounts WHERE id_account = ?")
            .bind(account)
            .fetch_one(&mut *connection)
            .await?;

    // Remove dust notes (too small to pay for a single logical action)
    let before_dust: [usize; NUM_POOLS] = std::array::from_fn(|p| input_pools[p].len());
    for pool in input_pools.iter_mut() {
        pool.retain(|n| n.amount >= COST_PER_ACTION);
    }
    info!(
        "plan: after dust filter — t:{}→{}, s:{}→{}, o:{}→{}, iw:{}→{}",
        before_dust[0], input_pools[0].len(),
        before_dust[1], input_pools[1].len(),
        before_dust[2], input_pools[2].len(),
        before_dust[3], input_pools[3].len(),
    );

    // Build asset→index lookup: 0 = ZEC, 1+ = index into zsa_assets
    let zsa_index: HashMap<[u8; 32], u8> = zsa_assets
        .iter()
        .enumerate()
        .map(|(i, a)| (*a, (i + 1) as u8))
        .collect();

    // Clone for move-capture in closures below
    let zi = zsa_index.clone();

    fn resolve_asset_index(
        asset_base: &Vec<u8>,
        zec_key: [u8; 32],
        zsa_index: &HashMap<[u8; 32], u8>,
    ) -> u8 {
        let asset_bytes: [u8; 32] = asset_base.clone().try_into().unwrap_or(zec_key);
        if asset_bytes == zec_key {
            0
        } else {
            zsa_index.get(&asset_bytes).copied().unwrap_or(0)
        }
    }

    // ── Coin selection via solve::select_notes ─────────────────────────────
    // Stamp asset_index on notes: 0 = ZEC, 1+ = index into zsa_assets
    let select_notes_input: Vec<solve::Note> = input_pools
        .iter()
        .enumerate()
        .flat_map(|(pool, notes)| {
            let zi = zi.clone();
            notes.iter().enumerate().map(move |(idx, n)| {
                let asset_index = resolve_asset_index(&n.asset_base, zec_key, &zi);
                solve::Note { pool: pool as u8, amount: n.amount, pool_index: idx, asset_index }
            })
        })
        .collect();

    // Compute pool preference per recipient once (explicit pool hint, or
    // fall back to the address-derived pool from decompose_address).
    let pool_prefs: Vec<u8> = recipients
        .iter()
        .zip(decomposed.iter())
        .map(|(r, dr)| {
            r.pools
                .and_then(|p| PoolMask(p).to_best_pool())
                .unwrap_or(dr.receiver.pool)
        })
        .collect();

    let select_outputs: Vec<solve::Output> = pool_prefs
        .iter()
        .zip(decomposed.iter())
        .map(|(&pool, dr)| {
            let asset_index = resolve_asset_index(&dr.asset_base, zec_key, &zsa_index);
            solve::Output { pool, amount: dr.amount, asset_index }
        })
        .collect();

    info!(
        "plan: calling select_notes — {} input notes, {} outputs, migration={}, recipient_pays_fee={}, first_recipient={}, mode={:?}",
        select_notes_input.len(), select_outputs.len(), migration, recipient_pays_fee,
        recipients.first().map(|r| r.amount).unwrap_or(0), mode
    );
    for o in &select_outputs {
        info!("plan: output pool={} amount={}", o.pool, o.amount);
    }

    let selection = solve::select_notes(
        &select_notes_input,
        &select_outputs,
        COST_PER_ACTION,
        migration,
        recipient_pays_fee,
        recipients.first().map(|r| r.amount).unwrap_or(0),
        mode,
    )
    .ok_or_else(|| anyhow!("No feasible note selection found"))?;

    info!(
        "plan: select_notes succeeded — fee={}, change_pool={}, selected_inputs={}",
        selection.fee, selection.change_pool, selection.inputs.len()
    );

    // Mark selected notes as fully consumed (select_notes uses 0/1 knapsack)
    for pool in 0..NUM_POOLS {
        for &idx in &selection.per_pool_indices[pool] {
            input_pools[pool][idx].remaining = 0;
        }
    }

    // ZSA assets only exist in Orchard; force change to orchard if any ZSA.
    // The ZEC change output satisfies ZIP-226 (no dummy needed).
    let change_pool = if has_zsa { 2 } else { selection.change_pool };

    // ── Compute ZSA change amounts ───────────────────────────────────────
    // Per-asset: sum of selected ZSA notes minus required ZSA outputs.
    let mut zsa_changes: Vec<([u8; 32], u64)> = vec![];
    if has_zsa {
        let mut zsa_selected: HashMap<[u8; 32], u64> = HashMap::new();
        // Pool 2 (Orchard) is where ZSA notes live
        for &idx in &selection.per_pool_indices[2] {
            let note = &input_pools[2][idx];
            let asset_bytes: [u8; 32] = note.asset_base.clone().try_into().unwrap_or(zec_key);
            if asset_bytes != zec_key {
                *zsa_selected.entry(asset_bytes).or_default() += note.amount;
            }
        }
        for asset in &zsa_assets {
            let selected = *zsa_selected.get(asset).unwrap_or(&0);
            let needed: u64 = decomposed
                .iter()
                .filter(|d| d.asset_base == asset.to_vec())
                .map(|d| d.amount)
                .sum();
            if selected > needed {
                zsa_changes.push((*asset, selected - needed));
            }
        }
    }

    // ── Build RecipientStates ────────────────────────────────────────────
    let mut recipient_states: Vec<RecipientState> = pool_prefs
        .iter()
        .zip(decomposed.iter())
        .map(|(&pool, dr)| {
            RecipientState {
                recipient: Recipient {
                    address: dr.address.clone(),
                    amount: dr.amount,
                    asset_base: dr.asset_base.clone(),
                    memo_bytes: dr.memo_bytes.clone(),
                    user_memo: dr.memo.clone(),
                    ..Default::default()
                },
                remaining: 0, // fully funded by select_notes
                pool_mask: PoolMask::from_pool(pool),
                asset_base: dr.asset_base.clone(),
            }
        })
        .collect();

    // Append ZSA change outputs (ZIP-226: ZEC outputs before ZSA outputs)
    for (asset, change_amount) in &zsa_changes {
        recipient_states.push(RecipientState {
            recipient: Recipient {
                address: String::new(), // filled in below with change_address
                amount: *change_amount,
                asset_base: asset.to_vec(),
                ..Recipient::default()
            },
            remaining: 0,
            pool_mask: PoolMask::from_pool(2), // ZSA always Orchard
            asset_base: asset.to_vec(),
        });
    }

    // ── Fee, totals, and change (select_notes already validated feasibility) ─
    // Issuance actions add separate logical actions on top of regular pool
    // actions (ZIP-233). First issuance: 2 notes (reference + real), reissuance: 1.
    let issuance_fee = issuance
        .map(|info| if info.first_issuance { 2 } else { 1 } * COST_PER_ACTION)
        .unwrap_or(0);
    let fee = selection.fee + issuance_fee;
    info!("Fee (select_notes + issuance): {}", to_zec(fee));

    // When the recipient pays the fee, deduct it from the first recipient
    // so the sender only needs to cover (total_output - fee), matching the
    // solver's target of `output_sum` (without fee).
    if recipient_pays_fee {
        if let Some(first) = recipient_states.first_mut() {
            first.recipient.amount = first.recipient.amount.saturating_sub(fee);
        }
    }

    let total_output: u64 = recipient_states.iter().map(|r| r.recipient.amount).sum();
    let total_input: u64 = selection.inputs.iter().map(|n| n.amount).sum();
    let change = total_input.saturating_sub(total_output + fee);

    info!(
        "change: {}, pool: {change_pool}, fee: {}",
        to_zec(change),
        to_zec(fee)
    );

    // ── Log outputs ──────────────────────────────────────────────────────
    for r in &recipient_states {
        info!(
            "address: {}, pool: {}, amount: {}",
            r.recipient.address,
            r.pool_mask.to_best_pool().unwrap(),
            to_zec(r.recipient.amount)
        );
    }

    // ── Fetch tree states and anchors ────────────────────────────────────
    let h = crate::sync::get_db_height(connection, account).await?;
    let (ts, to, ti) = crate::sync::get_tree_state(network, client, h.height).await?;
    let es = ts.to_edge(&SaplingHasher::default());
    let eo = to.to_edge(&OrchardHasher::default());
    let ei = ti.to_edge(&OrchardHasher::default());
    let sapling_anchor = es.root(&SaplingHasher::default());
    let orchard_anchor = eo.root(&OrchardHasher::default());
    let ironwood_anchor = ei.root(&OrchardHasher::default());

    // Determine which pools are active in this transaction
    let mut has_pool = [false; NUM_POOLS as usize];
    for pool in 1..NUM_POOLS {
        let p = pool as u8;
        has_pool[pool] = input_pools[pool].iter().any(|inp| inp.is_used())
            || recipient_states.iter().any(|r| r.pool_mask.to_best_pool() == Some(p))
            || change_pool == p;
    }
    has_pool[3] &= ironwood_active;
    // ZSA assets only exist in Orchard pool; ensure pool 2 is active
    // when ZSA is present (covers issuance-only case with no ZSA notes).
    has_pool[2] |= has_zsa;

    // ── Fetch change address ─────────────────────────────────────────────
    let change_scope = if use_internal { 1 } else { 0 };
    let mut change_address =
        get_account_full_address(network, connection, account, change_scope, hw).await?;
    let tkeys = select_account_transparent(connection, account, dindex).await?;
    if change_pool == 0 && tkeys.xvk.is_some() {
        change_address = generate_next_change_address(network, connection, account)
            .await?
            .unwrap();
    }

    // Fill in ZSA change output addresses
    for rs in &mut recipient_states {
        if rs.recipient.address.is_empty() && rs.asset_base != zec_key.to_vec() {
            rs.recipient.address = change_address.clone();
        }
    }

    // ── Fetch keys ───────────────────────────────────────────────────────
    let svk = get_sapling_vk(connection, account).await?;
    let ovk = get_orchard_vk(connection, account).await?;
    let ssk = get_sapling_sk(&mut *connection, account).await?;
    let osk = get_orchard_sk(&mut *connection, account).await?;

    // ── Build transaction ────────────────────────────────────────────────
    let current_height = client.latest_height().await?;
    let target_height = current_height;

    let build_config = BuildConfig::Standard {
        sapling_anchor: if has_pool[1] {
            sapling_crypto::Anchor::from_bytes(sapling_anchor).into_option()
        } else {
            None
        },
        orchard_anchor: if has_pool[2] {
            orchard::Anchor::from_bytes(orchard_anchor).into_option()
        } else {
            None
        },
        ironwood_anchor: if has_pool[3] {
            orchard::Anchor::from_bytes(ironwood_anchor).into_option()
        } else {
            None
        },
    };
    let mut builder = Builder::new(network, BlockHeight::from_u32(target_height), build_config);

    let es = es.to_auth_path(&SaplingHasher::default());
    let eo = eo.to_auth_path(&OrchardHasher::default());
    let ei = ei.to_auth_path(&OrchardHasher::default());
    let ers = empty_roots(&SaplingHasher::default());
    let ero = empty_roots(&OrchardHasher::default());

    let mut tsk_dindex = vec![];
    let mut s_scope = vec![];

    event!(Level::INFO, "Adding Inputs");

    let mut n_spends: [usize; NUM_POOLS as usize] = [0; NUM_POOLS as usize];
    let mut can_sign = true;

    for pool in input_pools.iter() {
        for inp in pool.iter() {
            if inp.is_used() {
                let InputNote {
                    id, amount, pool, ..
                } = inp;
                n_spends[*pool as usize] += 1;
                match pool {
                    0 => {
                        let row = sqlx::query(
                            "SELECT nullifier, t.pk, t.sk, t.scope, t.dindex, t.address, t.uncompressed FROM notes
                            JOIN transparent_address_accounts t ON notes.taddress = t.id_taddress
                            WHERE id_note = ?",
                        )
                        .bind(*id)
                        .fetch_one(&mut *connection)
                        .await?;

                        let _nf: Vec<u8> = row.get(0);
                        let pk: Vec<u8> = row.get(1);
                        let sk: Option<Vec<u8>> = row.get(2);
                        let scope: u32 = row.get(3);
                        let dindex_t: u32 = row.get(4);
                        let taddress: String = row.get(5);
                        let uncompressed: bool = row.get(6);

                        if sk.is_none() {
                            can_sign = false;
                        }

                        let pubkey = PublicKey::from_slice(&pk).unwrap();
                        let mut hash = [0u8; 32];
                        hash.copy_from_slice(&_nf[0..32]);
                        let n = u32::from_le_bytes(_nf[32..36].try_into().unwrap());
                        let utxo = OutPoint::new(hash, n);
                        let pk_bytes = if uncompressed {
                            pubkey.serialize_uncompressed().to_vec()
                        } else {
                            pubkey.serialize().to_vec()
                        };
                        let pkh: [u8; 20] =
                            Ripemd160::digest(Sha256::digest(&pk_bytes)).into();
                        let addr = TransparentAddress::PublicKeyHash(pkh);
                        let coin = TxOut::new(
                            Zatoshis::from_u64(*amount).unwrap(),
                            addr.script().into(),
                        );

                        builder
                            .add_transparent_input(
                                TransparentInputInfo::from_parts(
                                    utxo,
                                    coin,
                                    SpendInfo::P2pkh { pubkey },
                                )
                                .map_err(|e: zcash_transparent::builder::Error| anyhow!(e))?,
                            );
                        tsk_dindex.push((pubkey, scope, dindex_t, taddress, uncompressed));
                    }
                    1 => {
                        let (note, scope, merkle_path) = get_sapling_note(
                            connection,
                            *id,
                            h.height,
                            svk.as_ref().unwrap(),
                            &es,
                            &ers,
                        )
                        .await?;

                        if ssk.is_none() {
                            can_sign = false;
                        }

                        let dfvk = svk.as_ref().unwrap();
                        let fvk = dfvk.to_fvk(scope);
                        builder.add_sapling_spend::<Infallible>(fvk, note, merkle_path)?;
                        s_scope.push(scope);
                    }
                    2 => {

                        let (note, merkle_path) = get_orchard_note(
                            connection,
                            *id,
                            h.height,
                            ovk.as_ref().unwrap(),
                            &eo,
                            &ero,
                            orchard::NoteVersion::V2,
                        )
                        .await?;

                        if osk.is_none() {
                            can_sign = false;
                        }

                        builder.add_orchard_spend::<Infallible>(
                            ovk.clone().unwrap(),
                            note,
                            merkle_path,
                        )?;
                    }
                    3 => {
                        let (note, merkle_path) = get_orchard_note(
                            connection,
                            *id,
                            h.height,
                            ovk.as_ref().unwrap(),
                            &ei,
                            &ero,
                            orchard::NoteVersion::V3,
                        )
                        .await?;

                        if osk.is_none() {
                            can_sign = false;
                        }

                        builder.add_ironwood_spend::<Infallible>(
                            ovk.clone().unwrap(),
                            note,
                            merkle_path,
                        )?;
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    // ── Add outputs ──────────────────────────────────────────────────────
    event!(Level::INFO, "Adding Outputs");
    let mut n_outputs: [usize; NUM_POOLS as usize] = [0; NUM_POOLS as usize];

    for r in &recipient_states {
        let pool = r.pool_mask.to_best_pool().unwrap();
        let value = Zatoshis::from_u64(r.recipient.amount)?;
        let memo = encode_memo(&r.recipient)?.unwrap_or(MemoBytes::empty());

        n_outputs[pool as usize] += 1;
        match pool {
            0 => {
                if value != Zatoshis::ZERO {
                    let to = get_transparent_address(network, &r.recipient.address)?;
                    builder
                        .add_transparent_output(&to, value)
                        .map_err(|e: zcash_transparent::builder::Error| anyhow!(e))?;
                }
            }
            1 => {
                let to = get_sapling_address(network, &r.recipient.address)?;
                builder.add_sapling_output::<Infallible>(
                    svk.as_ref().map(|svk| svk.to_ovk(Scope::External)),
                    to,
                    value,
                    memo,
                )?;
            }
            2 => {
                let to = get_orchard_address(network, &r.recipient.address)?;
                let asset_base = if r.asset_base == [0u8; 32].to_vec() {
                    AssetBase::zatoshi()
                } else {
                    let asset_bytes: [u8; 32] = r.asset_base.clone().try_into().map_err(
                        |v: Vec<u8>| anyhow!("Invalid asset_base length: expected 32, got {}", v.len()),
                    )?;
                    Option::from(AssetBase::from_bytes(&asset_bytes))
                        .ok_or_else(|| anyhow!("Invalid asset_base bytes: {}", hex::encode(&asset_bytes)))?
                };
                if ironwood_active {
                    // O->O self-send: use change output to avoid dummy-spend
                    // fee inflation (Orchard V3 disables cross-address transfers).
                    if let Some(ref fvk) = ovk {
                        builder.add_orchard_change_output::<Infallible>(
                            fvk.clone(),
                            Some(fvk.to_ovk(Scope::External)),
                            to,
                            value,
                            asset_base,
                            MemoBytes::empty(),
                        )?;
                    } else {
                        anyhow::bail!("No orchard key for migration change output");
                    }
                } else {
                    builder.add_orchard_output::<Infallible>(
                        ovk.as_ref().map(|ovk| ovk.to_ovk(Scope::External)),
                        to,
                        value,
                        asset_base,
                        memo,
                    )?;
                }
            }
            3 => {
                let to = get_orchard_address(network, &r.recipient.address)?;
                builder.add_ironwood_output::<Infallible>(
                    ovk.as_ref().map(|ovk| ovk.to_ovk(Scope::External)),
                    to,
                    value,
                    memo,
                )?;
            }
            _ => {}
        }
    }

    // ── Add change output ────────────────────────────────────────────────
    if change > 0 {
        let change_addr = if change_pool == 0 && tkeys.xvk.is_some() {
            generate_next_change_address(network, connection, account)
                .await?
                .unwrap()
        } else {
            change_address.clone()
        };
        match change_pool {
            0 => {
                let to = get_transparent_address(network, &change_addr)?;
                builder
                    .add_transparent_output(&to, Zatoshis::const_from_u64(change))
                    .map_err(|e: zcash_transparent::builder::Error| anyhow!(e))?;
            }
            1 => {
                let to = get_sapling_address(network, &change_addr)?;
                builder.add_sapling_output::<Infallible>(
                    svk.as_ref().map(|svk| svk.to_ovk(Scope::External)),
                    to,
                    Zatoshis::const_from_u64(change),
                    MemoBytes::empty(),
                )?;
            }
            2 => {
                let to = get_orchard_address(network, &change_addr)?;
                if ironwood_active {
                    if let Some(ref fvk) = ovk {
                        builder.add_orchard_change_output::<Infallible>(
                            fvk.clone(),
                            Some(fvk.to_ovk(Scope::External)),
                            to,
                            Zatoshis::const_from_u64(change),
                            AssetBase::zatoshi(),
                            MemoBytes::empty(),
                        )?;
                    } else {
                        anyhow::bail!("No orchard key for change output");
                    }
                } else {
                    builder.add_orchard_output::<Infallible>(
                        ovk.as_ref().map(|ovk| ovk.to_ovk(Scope::External)),
                        to,
                        Zatoshis::const_from_u64(change),
                        AssetBase::zatoshi(),
                        MemoBytes::empty(),
                    )?;
                }
            }
            3 => {
                let to = get_orchard_address(network, &change_addr)?;
                if let Some(ref fvk) = ovk {
                    builder.add_ironwood_output::<Infallible>(
                        Some(fvk.to_ovk(Scope::External)),
                        to,
                        Zatoshis::const_from_u64(change),
                        MemoBytes::empty(),
                    )?;
                } else {
                    anyhow::bail!("No orchard key for ironwood change output");
                }
            }
            _ => {}
        }
    }

    // ── Build PCZT ───────────────────────────────────────────────────────
    info!("Building");
    event!(Level::INFO, "Preparing PCZT");

    // Attach ZsaBuilder before build_for_pczt for fee computation (ZIP-317)
    if let Some(info) = issuance {
        let oaddress = ovk
            .as_ref()
            .ok_or_else(|| anyhow!("No orchard key for issuance"))?
            .address_at(dindex, Scope::External);
        let zsa = build_zsa_builder(info, oaddress)?;
        builder.set_zsa_builder(zsa);
    }

    let r = builder.build_for_pczt(OsRng, &FeeRule::standard(), |_asset: &AssetBase| false)?;
    let sapling_meta = &r.sapling_meta;
    let orchard_meta = &r.orchard_meta;
    let ironwood_meta = &r.ironwood_meta;

    let pczt = Creator::build_from_parts(r.pczt_parts).unwrap();
    info!("Created");

    let updater = Updater::new(pczt);
    let updater = updater
        .update_transparent_with(|mut u| {
            for (i, (pubkey, scope, dindex_t, taddress, uncompressed)) in
                tsk_dindex.into_iter().enumerate()
            {
                u.update_input_with(i, |mut u| {
                    let derivation_path = vec![scope, dindex_t];
                    let path = Bip32Derivation::parse([0u8; 32], derivation_path).unwrap();
                    u.set_bip32_derivation(pubkey.serialize(), path);
                    u.set_proprietary("scope".to_string(), scope.to_le_bytes().to_vec());
                    u.set_proprietary("dindex".to_string(), dindex_t.to_le_bytes().to_vec());
                    u.set_proprietary("address".to_string(), taddress.into_bytes());
                    u.set_proprietary("uncompressed".to_string(), vec![uncompressed as u8]);
                    let pk_bytes = if uncompressed {
                        pubkey.serialize_uncompressed().to_vec()
                    } else {
                        pubkey.serialize().to_vec()
                    };
                    u.set_hash160_preimage(pk_bytes);
                    Ok(())
                })?;
            }
            Ok(())
        })
        .unwrap();

    let updater = updater
        .update_sapling_with(|mut u| {
            for (c_input, scope) in s_scope.iter().enumerate() {
                let bundle_index = sapling_meta.spend_index(c_input).unwrap();
                u.update_spend_with(bundle_index, |mut u| {
                    u.set_proprietary("scope".to_string(), scope.to_le_bytes().to_vec());
                    Ok(())
                })?;
            }
            Ok(())
        })
        .unwrap();

    let pczt = updater.finish();

    // Issuer phase 1: build the AwaitingSighash issue bundle
    let pczt = if let Some(info) = issuance {
        let oaddress = ovk
            .as_ref()
            .ok_or_else(|| anyhow!("No orchard key for issuance"))?
            .address_at(dindex, Scope::External);
        let zsa_builder = build_zsa_builder(info, oaddress)?;
        Issuer::new(pczt)
            .build_awaiting_sighash(zsa_builder, OsRng)
            .map_err(|e| anyhow!("Issuer (phase 1) failed: {e:?}"))?
    } else {
        pczt
    };

    let (pczt, shielded_sighash) = IoFinalizer::new(pczt).finalize_io().unwrap();
    info!("IO Finalized");

    // Issuer phase 2: sign the issue bundle
    let pczt = if let Some(info) = issuance {
        Issuer::new(pczt)
            .sign(&info.isk, shielded_sighash)
            .map_err(|e| anyhow!("Issuer (phase 2/sign) failed: {e:?}"))?
    } else {
        pczt
    };

    let n_orchard_actions = pczt.orchard().actions().len();
    let pczt_package = PcztPackage {
        pczt: pczt.serialize().unwrap(),
        n_spends: [n_spends[0], n_spends[1], n_spends[2], n_spends[3]],
        sapling_indices: (0..n_spends[1])
            .map(|n| sapling_meta.spend_index(n).unwrap())
            .collect(),
        orchard_indices: if ironwood_active {
            (0..n_orchard_actions).collect()
        } else {
            (0..n_spends[2])
                .map(|n| orchard_meta.spend_action_index(n).unwrap())
                .collect()
        },
        ironwood_indices: (0..n_spends[3])
            .map(|n| ironwood_meta.spend_action_index(n).unwrap())
            .collect(),
        can_sign,
        can_broadcast: false,
        price,
        category,
        is_issuance: issuance.is_some(),
    };

    Ok(pczt_package)
}
fn encode_memo(recipient: &Recipient) -> Result<Option<MemoBytes>> {
    let text_memo = recipient
        .user_memo
        .as_ref()
        .map(|s| Memo::from_str(s))
        .transpose()?
        .map(MemoBytes::from);
    let byte_memo = recipient
        .memo_bytes
        .as_ref()
        .map(|mb| MemoBytes::from_bytes(mb))
        .transpose()?;
    let memo = text_memo.or(byte_memo);
    Ok(memo)
}

pub async fn sign_transaction(
    connection: &mut SqliteConnection,
    account: u32,
    network: &crate::api::coin::Network,
    pczt: &PcztPackage,
) -> Result<PcztPackage> {
    let span = span!(Level::INFO, "transaction");

    let PcztPackage {
        pczt,
        n_spends,
        sapling_indices,
        orchard_indices,
        ironwood_indices,
        price,
        category,
        is_issuance,
        ..
    } = pczt;
    let pczt = Pczt::parse(pczt).unwrap();

    let ironwood_active = network.is_nu_active(
        NetworkUpgrade::Nu6_3,
        BlockHeight::from_u32(*pczt.global().expiry_height()),
    );

    let dindex = get_account_dindex(connection, account).await?;
    let tkeys = select_account_transparent(connection, account, dindex).await?;
    let tsk = tkeys.xsk;
    let ssk = get_sapling_sk(connection, account).await?;
    let osk = get_orchard_sk(connection, account).await?;
    let osak = osk.map(|osk| SpendAuthorizingKey::from(&osk));

    let updater = Updater::new(pczt);
    let pgk = ssk.clone().map(|ssk| ssk.expsk.proof_generation_key());
    let internal_pgk = ssk
        .clone()
        .map(|ssk| ssk.derive_internal().expsk.proof_generation_key());
    let updater = updater
        .update_sapling_with(|mut u| {
            for bundle_index in sapling_indices.iter() {
                let spend = &u.bundle().spends()[*bundle_index];
                let scope =
                    u32::from_le_bytes(spend.proprietary()["scope"].clone().try_into().unwrap());
                u.update_spend_with(*bundle_index, |mut u| {
                    u.set_proof_generation_key(sapling_pgk_for_scope(
                        scope,
                        pgk.clone().expect("proof_generation_key"),
                        internal_pgk.clone().expect("internal_proof_generation_key"),
                    ))
                    .unwrap();

                    Ok(())
                })
                .unwrap();
            }
            Ok(())
        })
        .unwrap();
    let pczt = updater.finish();
    info!("Updated");

    let mut signer = Signer::new(pczt.clone()).unwrap();
    let tbundle = pczt.transparent();
    let sbundle = pczt.sapling();
    for index in 0..n_spends[0] {
        info!("signing transparent {index}");
        let inp = &tbundle.inputs()[index];
        let scope = u32::from_le_bytes(inp.proprietary()["scope"].clone().try_into().unwrap());
        let dindex = u32::from_le_bytes(inp.proprietary()["dindex"].clone().try_into().unwrap());
        // Check if "uncompressed" flag exists in proprietary, default to false (compressed)
        let uncompressed_flag = if let Some(val) = inp.proprietary().get("uncompressed") {
            if !val.is_empty() {
                val[0] != 0
            } else {
                info!(
                    "Invalid uncompressed flag length: {}, defaulting to compressed",
                    val.len()
                );
                false
            }
        } else {
            info!("No 'uncompressed' proprietary field found, defaulting to compressed");
            false
        };
        info!(
            "Signing transparent input {}: scope={}, dindex={}, uncompressed={}",
            index, scope, dindex, uncompressed_flag
        );

        // Get the signing key
        let sk = match tsk.as_ref() {
            // From the derivation path if we have the xsk
            Some(tsk) => {
                let sk = derive_transparent_sk(tsk, scope, dindex)?;
                SecretKey::from_bytes(&sk.try_into().unwrap()).ok()
            }
            // Or directly from the private key
            None => {
                let address = String::from_utf8(inp.proprietary()["address"].clone())?;
                sqlx::query(
                    "SELECT sk FROM transparent_address_accounts
                    WHERE account = ?1 AND address = ?2",
                )
                .bind(account)
                .bind(&address)
                .map(|r| {
                    let sk: Vec<u8> = r.get(0);
                    SecretKey::from_bytes(&sk.try_into().unwrap()).unwrap()
                })
                .fetch_optional(&mut *connection)
                .await?
            }
        };
        let sk = sk.ok_or(Error::NoSigningKey)?;

        // Derive pubkey from secret key to check
        let secp = secp256k1::Secp256k1::new();
        let derived_pubkey = secp256k1::PublicKey::from_secret_key(&secp, &sk);
        let derived_compressed = derived_pubkey.serialize();
        let derived_uncompressed = derived_pubkey.serialize_uncompressed();
        let hash_compressed = zcash_transparent::util::hash160::hash(&derived_compressed);
        let hash_uncompressed = zcash_transparent::util::hash160::hash(&derived_uncompressed);
        info!(
            "Derived pubkey (compressed): hash={}, len={}",
            hex::encode(hash_compressed),
            derived_compressed.len()
        );
        info!(
            "Derived pubkey (uncompressed): hash={}, len={}",
            hex::encode(hash_uncompressed),
            65
        );

        // Get the sighash and sign manually
        let sighash = signer.transparent_sighash(index).unwrap();
        let msg = secp256k1::Message::from_digest(sighash);
        let sig = secp.sign_ecdsa(&msg, &sk);

        // Append the signature - the pubkey will be retrieved from hash160_preimages
        info!("Appending signature for input {}", index);
        match signer.append_transparent_signature(index, sig) {
            Ok(_) => info!("Successfully appended signature"),
            Err(e) => info!("Failed to append signature: {:?}", e),
        }
    }
    for (index, bundle_index) in sapling_indices.iter().enumerate() {
        info!("signing sapling {index}");
        let spend = &sbundle.spends()[*bundle_index];
        let scope = u32::from_le_bytes(spend.proprietary()["scope"].clone().try_into().unwrap());
        let ssk = ssk.as_ref().map(|ssk| sapling_ssk_for_scope(scope, ssk));
        let Some(sk) = ssk.as_ref().map(|sk| &sk.expsk.ask) else {
            return Err(Error::NoSigningKey.into());
        };
        signer.sign_sapling(*bundle_index, sk).unwrap();
    }
    for (index, bundle_index) in orchard_indices.iter().enumerate() {
        info!("signing orchard {index}");
        let Some(osak) = osak.as_ref() else {
            return Err(Error::NoSigningKey.into());
        };
        signer.sign_orchard(*bundle_index, osak).unwrap();
    }
    for (index, bundle_index) in ironwood_indices.iter().enumerate() {
        info!("signing ironwood {index}");
        let Some(osak) = osak.as_ref() else {
            return Err(Error::NoSigningKey.into());
        };
        signer.sign_ironwood(*bundle_index, osak).unwrap();
    }
    let pczt = signer.finish();

    span.in_scope(|| {
        info!("Adding Proofs to PCZT");
    });
    let sapling_prover = get_sapling_prover().await?;

    let orchard_pk = get_orchard_pk(network, ironwood_active);
    let pczt = Prover::new(pczt)
        .create_sapling_proofs(sapling_prover, sapling_prover)
        .unwrap()
        .create_orchard_proof(orchard_pk)
        .unwrap()
        .create_ironwood_proof(&IRONWOOD_PK)
        .unwrap()
        .finish();
    info!("Proved");

    let pczt = SpendFinalizer::new(pczt).finalize_spends().unwrap();
    info!("Spend Finalized");

    Ok(PcztPackage {
        pczt: pczt.serialize().unwrap(),
        n_spends: *n_spends,
        sapling_indices: sapling_indices.clone(),
        orchard_indices: orchard_indices.clone(),
        ironwood_indices: ironwood_indices.clone(),
        can_sign: true,
        can_broadcast: true,
        price: *price,
        category: *category,
        is_issuance: *is_issuance,
    })
}

pub async fn extract_transaction(package: &PcztPackage) -> Result<Vec<u8>> {
    let span = span!(Level::INFO, "transaction");
    span.in_scope(|| {
        info!("Extracting Tx");
    });

    let pczt = Pczt::parse(&package.pczt).unwrap();

    let sapling_prover = get_sapling_prover().await?;
    let (svk, ovk) = sapling_prover.verifying_keys();
    let tx_extractor = TransactionExtractor::new(pczt).with_sapling(&svk, &ovk);
    match tx_extractor.extract() {
        Ok(tx) => {
            if let Some(bundle) = tx.sapling_bundle() {
                let vb: i64 = (*bundle.value_balance()).into();
                info!(
                    "Sapling verify OK: spends={} outputs={} valueBalance={}",
                    bundle.shielded_spends().len(),
                    bundle.shielded_outputs().len(),
                    vb
                );
            }
            let mut tx_bytes = vec![];
            tx.write(&mut tx_bytes).unwrap();
            info!("Tx Extracted");
            span.in_scope(|| {
                info!("TX HEX: {}", hex::encode(&tx_bytes));
                info!("Tx Ready - {} bytes", tx_bytes.len());
            });
            return Ok(tx_bytes);
        }
        Err(e) => {
            info!("Extraction failed: {:?}", e);
            return Err(anyhow!("Extraction failed: {:?}", e));
        }
    }
}

struct MyTransparentAddress(TransparentAddress);
impl TryFromAddress for MyTransparentAddress {
    type Error = ();

    fn try_from_unified(
        _net: NetworkType,
        data: zcash_address::unified::Address,
    ) -> std::result::Result<Self, ConversionError<Self::Error>> {
        let ua = UnifiedAddress::try_from(data).unwrap();
        ua.transparent()
            .map(|v| MyTransparentAddress(*v))
            .ok_or(ConversionError::User(()))
    }

    fn try_from_transparent_p2pkh(
        _net: NetworkType,
        data: [u8; 20],
    ) -> Result<Self, ConversionError<Self::Error>> {
        Ok(MyTransparentAddress(TransparentAddress::PublicKeyHash(
            data,
        )))
    }

    fn try_from_tex(
        _net: NetworkType,
        data: [u8; 20],
    ) -> std::result::Result<Self, ConversionError<Self::Error>> {
        Ok(MyTransparentAddress(TransparentAddress::PublicKeyHash(
            data,
        )))
    }

    fn try_from_transparent_p2sh(
        _net: NetworkType,
        data: [u8; 20],
    ) -> std::result::Result<Self, ConversionError<Self::Error>> {
        Ok(MyTransparentAddress(TransparentAddress::ScriptHash(data)))
    }
}

fn get_transparent_address(network: &Network, address: &str) -> Result<TransparentAddress> {
    let addr = ZcashAddress::try_from_encoded(address)?;
    if addr.can_receive_as(zcash_protocol::PoolType::Transparent) {
        let taddr: MyTransparentAddress = addr.convert_if_network(network.network_type()).unwrap();
        return Ok(taddr.0);
    }
    anyhow::bail!("Invalid transparent address: {address}");
}

fn get_sapling_address(network: &Network, address: &str) -> Result<PaymentAddress> {
    if let Ok(addr) = PaymentAddress::decode(network, address) {
        return Ok(addr);
    }
    if let Ok(addr) = UnifiedAddress::decode(network, address) {
        let addr = addr.sapling().unwrap();
        Ok(*addr)
    } else {
        anyhow::bail!("Invalid sapling address: {address}");
    }
}

fn get_orchard_address(network: &Network, address: &str) -> Result<Address> {
    if let Ok(addr) = UnifiedAddress::decode(network, address) {
        let addr = addr.orchard().unwrap();
        Ok(*addr)
    } else {
        anyhow::bail!("Invalid orchard address: {address}");
    }
}


pub async fn fetch_unspent_notes_grouped_by_pool(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<Vec<InputNote>> {
    let unspent_notes = sqlx::query(
        "SELECT a.id_note, a.height, a.pool, a.value, a.id_asset, a.taddress,
                COALESCE(ast.asset_base, X'0000000000000000000000000000000000000000000000000000000000000000') as asset_base
        FROM notes a
        LEFT JOIN spends b ON a.id_note = b.id_note
        LEFT JOIN assets ast ON a.id_asset = ast.id_asset
        WHERE b.id_note IS NULL AND a.account = ?
        AND locked = 0
        ORDER BY a.pool",
    )
    .bind(account)
    .map(|row: SqliteRow| {
        let id_note: u32 = row.get(0);
        let height: u32 = row.get(1);
        let pool: u8 = row.get(2);
        let value: i64 = row.get(3);
        let id_asset: Option<i64> = row.get(4);
        let taddress: Option<i64> = row.get(5);
        let asset_base: Vec<u8> = row.get(6);
        InputNote {
            id: id_note,
            height,
            amount: value as u64,
            remaining: value as u64,
            pool,
            id_asset: id_asset.map(|v| v as u32),
            asset_base,
            taddress: taddress.map(|v| v as u32),
        }
    })
    .fetch_all(connection)
    .await?;

    Ok(unspent_notes)
}

pub async fn fetch_unspent_notes_by_pool(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<Vec<Vec<InputNote>>> {
    let unspent_notes = sqlx::query(
        "SELECT a.id_note, a.height, a.pool, a.value, a.id_asset, a.taddress,
                COALESCE(ast.asset_base, X'0000000000000000000000000000000000000000000000000000000000000000') as asset_base
        FROM notes a
        LEFT JOIN spends b ON a.id_note = b.id_note
        LEFT JOIN assets ast ON a.id_asset = ast.id_asset
        WHERE b.id_note IS NULL AND a.account = ?
        AND locked = 0",
    )
    .bind(account)
    .map(|row: SqliteRow| {
        let id_note: u32 = row.get(0);
        let height: u32 = row.get(1);
        let pool: u8 = row.get(2);
        let value: i64 = row.get(3);
        let id_asset: Option<i64> = row.get(4);
        let taddress: Option<i64> = row.get(5);
        let asset_base: Vec<u8> = row.get(6);
        InputNote {
            id: id_note,
            height,
            amount: value as u64,
            remaining: value as u64,
            pool,
            id_asset: id_asset.map(|v| v as u32),
            asset_base,
            taddress: taddress.map(|v| v as u32),
        }
    })
    .fetch_all(connection)
    .await?;

    let mut result: Vec<Vec<InputNote>> = vec![vec![]; NUM_POOLS as usize];
    for note in unspent_notes {
        let pool = note.pool as usize;
        anyhow::ensure!(pool < NUM_POOLS, "unexpected pool {pool}");
        result[pool].push(note);
    }
    Ok(result)
}

pub async fn get_sapling_prover() -> Result<&'static LocalTxProver> {
    static PROVER: tokio::sync::OnceCell<LocalTxProver> = tokio::sync::OnceCell::const_new();
    PROVER
        .get_or_try_init(|| async {
            let params_dir = crate::api::sapling::resolve_params_dir()
                .ok_or_else(|| anyhow::anyhow!("Failed to resolve Sapling parameters directory"))?;
            let spend_path = params_dir.join(zcash_proofs::SAPLING_SPEND_NAME);
            let output_path = params_dir.join(zcash_proofs::SAPLING_OUTPUT_NAME);

            if spend_path.exists() && output_path.exists() {
                return Ok(LocalTxProver::new(&spend_path, &output_path));
            }
            // Parameters not found on disk — download them.
            crate::api::sapling::download_sapling_params().await?;
            Ok(LocalTxProver::new(&spend_path, &output_path))
        })
        .await
}
pub static ORCHARD_VANILLA_PK: LazyLock<ProvingKey> =
    LazyLock::new(|| ProvingKey::build(orchard::circuit::OrchardCircuitVersion::FixedPostNu6_2));
pub static ORCHARD_ZSA_PK: LazyLock<ProvingKey> =
    LazyLock::new(|| ProvingKey::build(orchard::circuit::OrchardCircuitVersion::ZsaFixed));
pub static IRONWOOD_PK: LazyLock<ProvingKey> =
    LazyLock::new(|| ProvingKey::build(orchard::circuit::OrchardCircuitVersion::PostNu6_3));

pub fn get_orchard_pk(
    network: &crate::api::coin::Network,
    ironwood_active: bool,
) -> &'static ProvingKey {
    // ZSA and Ironwood are mutually exclusive hard forks with different
    // V6 version group IDs and circuit versions.
    let uses_orchard_zsa = match network {
        crate::api::coin::Network::Regtest(config)
        | crate::api::coin::Network::ZsaRegtest(config) => {
            config.orchard_mode() == zcash_protocol::consensus::OrchardMode::Zsa
        }
        _ => false,
    };

    if uses_orchard_zsa {
        &ORCHARD_ZSA_PK
    } else if ironwood_active {
        &IRONWOOD_PK
    } else {
        &ORCHARD_VANILLA_PK
    }
}
