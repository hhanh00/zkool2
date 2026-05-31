use std::{collections::HashMap, convert::Infallible, str::FromStr as _, sync::LazyLock};

use anyhow::{anyhow, Result};

use bip32::PrivateKey;
use itertools::Itertools;
use orchard::{
    circuit::ProvingKey,
    flavor::OrchardZSA,
    keys::{Scope, SpendAuthorizingKey},
    note::{AssetBase, ExtractedNoteCommitment},
    Address,
};
use pczt::{
    roles::{
        creator::Creator, io_finalizer::IoFinalizer, prover::Prover, signer::Signer,
        spend_finalizer::SpendFinalizer, tx_extractor::TransactionExtractor, updater::Updater,
    },
    Pczt,
};
use rand_core::{OsRng, RngCore};
use ripemd::Ripemd160;
use sapling_crypto::{keys::FullViewingKey, zip32::DiversifiableFullViewingKey, PaymentAddress};
use secp256k1::{PublicKey, SecretKey};
use sha2::{Digest as _, Sha256};
use sqlx::{sqlite::SqliteRow, Row, SqliteConnection};
use tracing::{debug, event, info, span, Level};
use zcash_address::{ConversionError, TryFromAddress, ZcashAddress};
use zcash_keys::{address::UnifiedAddress, encoding::AddressCodec as _};
use zcash_primitives::transaction::{
    builder::{BuildConfig, Builder},
    fees::zip317::FeeRule,
};
use zcash_proofs::prover::LocalTxProver;
use zcash_protocol::{
    consensus::{BlockHeight, NetworkType, Parameters, ZIP212_GRACE_PERIOD},
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
    api::{coin::Network, pay::PcztPackage},
    db::{get_account_dindex, get_account_hw, select_account_transparent},
    pay::{
        error::Error,
        fee::{FeeManager, COST_PER_ACTION},
        pool::{PoolMask, ALL_POOLS},
        prepare::to_zec,
        InputNote, Recipient, RecipientState, TxPlanIn, TxPlanOut,
    },
    warp::hasher::{empty_roots, OrchardHasher, SaplingHasher},
    Client,
};

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
) -> Result<PcztPackage> {
    let span = span!(Level::INFO, "transaction");
    span.in_scope(|| {
        info!("Computing plan");
    });

    let dindex = get_account_dindex(connection, account).await?;
    let mut total_amount = 0;
    let mut total_fiat = 0.0;
    for r in recipients {
        if let Some(price) = r.price {
            total_fiat += price * r.amount as f64;
            total_amount += r.amount;
        }
    }
    let price = if total_amount != 0 {
        Some(total_fiat / total_amount as f64)
    } else {
        None
    };

    let has_tex = recipients
        .iter()
        .any(|r| is_tex(network, &r.address).unwrap_or_default());
    info!("has_tex: {account} {has_tex}");

    let mut can_sign = true;
    let hw = get_account_hw(&mut *connection, account).await?;
    let (use_internal,): (bool,) =
        sqlx::query_as("SELECT use_internal FROM accounts WHERE id_account = ?")
            .bind(account)
            .fetch_one(&mut *connection)
            .await?;

    // Pre-fetch the orchard change address. Used for dummy ZEC outputs
    // and later for actual change outputs.
    let change_scope = if use_internal { 1 } else { 0 };
    let change_address =
        get_account_full_address(network, connection, account, change_scope, hw).await?;

    let effective_src_pools = if has_tex || smart_transparent {
        PoolMask::from_pool(0) // restrict to transparent pool
    } else {
        crate::pay::plan::get_effective_src_pools(&mut *connection, account, src_pools).await?
    };

    let recipients = recipients.to_vec();
    let mut recipient_pools = PoolMask(0);
    for recipient in recipients.iter() {
        let pool = PoolMask::from_address(&recipient.address)?
            .intersect(&PoolMask(recipient.pools.unwrap_or(ALL_POOLS)));
        recipient_pools = recipient_pools.union(&pool);
    }
    info!(
        "effective_src_pools: {src_pools} {:#b}",
        effective_src_pools.0
    );
    info!("recipient_pools: {:#b}", recipient_pools.0);
    let change_pool = get_change_pool(effective_src_pools, recipient_pools);
    debug!("change_pool: {:#b}", change_pool);

    let mut fee_manager = FeeManager::default();
    fee_manager.add_output(change_pool);

    let confirmations = confirmations.unwrap_or_default();
    let height = client.latest_height().await?;
    let max_height = height.saturating_sub(confirmations);

    let mut input_pools = vec![vec![]; 3];
    let (inputs, recipients, recipient_pays_fee) = if smart_transparent {
        // Restrict to using one transparent address per shielding
        let mut notes = fetch_one_taddr_unspent_notes(connection, account).await?;
        notes.retain(|n| n.height <= max_height);
        // override the amount to the maximum amount available
        let max = notes.iter().map(|n| n.amount).sum::<u64>();
        let recipient = Recipient {
            amount: max,
            ..recipients.first().cloned().unwrap_or_default()
        };
        (notes, vec![recipient], true)
    } else {
        let mut notes = fetch_unspent_notes_grouped_by_pool(connection, account).await?;
        notes.retain(|n| n.height <= max_height);
        (
            notes,
            recipients,
            recipient_pays_fee,
        )
    };

    let mut recipient_states = recipients
        .into_iter()
        .map(|r| RecipientState::new(r).unwrap())
        .collect::<Vec<_>>();

    let zec_key = [0u8; 32].to_vec();

    // If ZSA assets are involved but there will be no ZEC action in the
    // orchard bundle (no ZEC recipient AND no ZEC notes to spend), add a
    // dummy ZEC recipient with amount 0. This ensures the orchard bundle
    // has at least one ZEC action (required by ZIP-226).
    let has_zsa = recipient_states.iter().any(|r| r.asset_base != zec_key);
    let has_zec_recipient = recipient_states.iter().any(|r| r.asset_base == zec_key);
    let has_zec_notes = input_pools[2].iter().any(|n: &InputNote| n.asset_base == zec_key);
    if has_zsa && !has_zec_recipient && !has_zec_notes {
        recipient_states.push(RecipientState {
            recipient: Recipient {
                address: change_address.clone(),
                amount: 0,
                asset_base: zec_key.clone(),
                ..Default::default()
            },
            remaining: 0,
            pool_mask: PoolMask::from_pool(2), // orchard
            asset_base: zec_key.clone(),
        });
    }

    // Ensure ZEC recipients come before ZSA recipients so the builder
    // processes them first, producing a ZEC-first action.
    recipient_states.sort_by_key(|r| r.asset_base != zec_key);

    debug!("Unspent notes:");
    for inp in inputs.iter() {
        debug!(
            "id: {}, pool: {}, amount: {}",
            inp.id,
            inp.pool,
            to_zec(inp.amount)
        );
    }

    // group the inputs by pool
    for (group, items) in inputs.into_iter().chunk_by(|inp| inp.pool).into_iter() {
        // skip if the pool is not in the source pools
        if effective_src_pools.0 & (1 << group) == 0 {
            continue;
        }
        input_pools[group as usize].extend(items);
    }

    // Split orchard notes (pool 2) by asset_base.
    // All orchard notes go into orchard_buckets for asset-aware fill.
    // ZEC orchard notes are referenced via orchard_buckets[zec_key] for fees.
    let mut orchard_buckets: HashMap<Vec<u8>, Vec<InputNote>> = HashMap::new();
    for note in input_pools[2].drain(..) {
        orchard_buckets
            .entry(note.asset_base.clone())
            .or_default()
            .push(note);
    }

    // we can merge notes from the same pool because they are fully fungible
    // but we should keep the funds from different pools separate
    // because even though they can participate in the same transaction
    // they don't have the same properties.
    // calculate_balance will return the balance for each pool
    // and we have to pick up notes to send to the recipients
    // There can be multiple recipients in the single transaction
    // Recipients can accept multiple receivers when they use a unified address
    // The simplest way to do this would be to choose any allowed receiver
    // and then pick up randomly notes from the wallet until we cover the
    // amount needed for the transaction
    // but this could be inefficient and leak information about the wallet
    // Instead we will choose based on the balances available and the
    // recipients
    //
    // We use two passes. In the first pass, we only consider the recipients
    // that have single receiver addresses. For these, there is no option
    // to choose the receiver. The only decision we need to make is to
    // choose what pool to use for the inputs.
    // This is handled by the function fill_single_receivers
    //
    let (mut single, mut double) = recipient_states
        .into_iter()
        .partition::<Vec<_>, _>(|r| r.pool_mask != PoolMask(6));

    // Account for per-asset orchard change outputs in the fee estimation.
    // Line 163 already counted one change output for the ZEC asset; add one
    // more for each unique non-ZEC asset among the recipients, since
    // per-asset change (lines 446-468) produces one output per asset.
    {
        let mut seen = std::collections::HashSet::new();
        for r in single.iter().chain(double.iter()) {
            if r.asset_base != zec_key && seen.insert(&r.asset_base) {
                fee_manager.add_output(change_pool);
            }
        }
    }

    // Pre-check: verify each recipient's asset bucket has sufficient balance
    for r in single.iter().chain(double.iter()) {
        let needed = r.remaining;
        if needed == 0 { continue; }
        let pool = r.pool_mask.to_best_pool().unwrap_or(2);
        if pool == 2 {
            let available: u64 = orchard_buckets.get(&r.asset_base)
                .map(|b| b.iter().map(|n| n.remaining).sum())
                .unwrap_or(0);
            if available < needed {
                let asset_label = if r.asset_base == zec_key {
                    "ZEC".to_string()
                } else if let Some(ref name) = r.recipient.asset_name {
                    name.clone()
                } else {
                    hex::encode(&r.asset_base)
                };
                return Err(Error::NotEnoughFunds(
                    format!("{} (need {}, have {})", asset_label, needed, available)
                ).into());
            }
        } else {
            let available: u64 = input_pools[pool as usize]
                .iter().map(|n| n.remaining).sum();
            if available < needed {
                return Err(Error::NotEnoughFunds(
                    format!("pool {}: need {}, have {}", pool, needed, available)
                ).into());
            }
        }
    }

    let mut fee_paid = 0;
    fill_single_receivers(
        &mut input_pools,
        &mut orchard_buckets,
        &mut single,
        &mut fee_manager,
        recipient_pays_fee,
        &mut fee_paid,
    )?;

    // In the second pass, we will consider the recipients that have
    // multiple receivers. We always favor shielded receivers over
    // transparent ones. Hence, if a UA has a transparent and a
    // sapling receiver, it counts as a single sapling receiver.
    // Then, the only time we can have a multiple receiver recipient
    // is when we have a sapling and an orchard receiver, ie.
    // when we have to choose between shielded pools

    let balances = input_pools
        .iter()
        .map(|pool| pool.iter().map(|n| n.remaining).sum::<u64>())
        .collect::<Vec<_>>();

    // In the second pass, we constrain the receiver to be the change pool
    // or the pool that we have the most balance in if the change pool is transparent
    // This is because we hope to minimize the amount that would have to go through the
    // turnstile.

    let largest_shielded_pool = if change_pool != 0 {
        PoolMask::from_pool(change_pool)
    } else if balances[1] > balances[2] {
        PoolMask(2)
    } else {
        PoolMask(4)
    };

    for d in double.iter_mut() {
        d.pool_mask = largest_shielded_pool;
    }

    fill_single_receivers(
        &mut input_pools,
        &mut orchard_buckets,
        &mut double,
        &mut fee_manager,
        recipient_pays_fee,
        &mut fee_paid,
    )?;

    // Now we have pick the inputs and paid the fee if the sender
    // should be paying it

    info!("Fee {}", &fee_manager);
    let fee = fee_manager.fee();

    if recipient_pays_fee {
        fee_paid += fee;
    }

    if fee > fee_paid {
        return Err(Error::NotEnoughFunds(to_zec(fee - fee_paid)).into());
    }

    let recipients = single.iter_mut().chain(double.iter_mut());
    for (i, r) in recipients.enumerate() {
        if r.remaining > 0 {
            let amt = if r.asset_base == zec_key {
                to_zec(r.remaining)
            } else if let Some(ref name) = r.recipient.asset_name {
                format!("{} {} (raw units)", r.remaining, name)
            } else {
                format!("{} (raw units, asset={})", r.remaining, hex::encode(&r.asset_base))
            };
            return Err(Error::NotEnoughFunds(amt).into());
        }
        if i == 0 && recipient_pays_fee {
            // if the recipient pays the fee, we need to pay it
            // from the first recipient
            if r.recipient.amount < fee {
                return Err(Error::NotEnoughFunds(to_zec(fee - r.recipient.amount)).into());
            }
            r.recipient.amount -= fee;
        }
    }

    // Total input from transparent + sapling
    let non_orchard_input: u64 = input_pools[0].iter()
        .map(|n| if n.is_used() { n.amount } else { 0 })
        .sum::<u64>()
        + input_pools[1].iter()
            .map(|n| if n.is_used() { n.amount } else { 0 })
            .sum::<u64>();
    // Total input from all orchard buckets
    let mut total_input: u64 = non_orchard_input;
    let mut orchard_per_asset: HashMap<Vec<u8>, (u64, u64)> = HashMap::new(); // (used_in, output) per asset
    for (asset_key, bucket) in orchard_buckets.iter() {
        let used: u64 = bucket.iter()
            .map(|n| if n.is_used() { n.amount } else { 0 })
            .sum::<u64>();
        total_input += used;
        orchard_per_asset.insert(asset_key.clone(), (used, 0));
    }

    let total_output: u64 = single.iter().chain(double.iter())
        .map(|r| r.recipient.amount).sum();
    // Track output per asset for change computation
    for r in single.iter().chain(double.iter()) {
        if r.pool_mask.to_best_pool().unwrap_or(2) == 2 {
            if let Some((_, ref mut out)) = orchard_per_asset.get_mut(&r.asset_base) {
                *out += r.recipient.amount;
            }
        }
    }

    let change = total_input.saturating_sub(total_output.saturating_add(fee));

    for o in single.iter_mut().chain(double.iter_mut()) {
        let RecipientState {
            recipient,
            remaining,
            pool_mask,
            ..
        } = o;
        assert_eq!(*remaining, 0);
        debug!(
            "address: {}, pool: {}, amount: {}",
            recipient.address,
            pool_mask.to_best_pool().unwrap(),
            to_zec(recipient.amount)
        );
    }

    info!(
        "change: {}, pool: {change_pool}, fee: {}",
        to_zec(change),
        to_zec(fee)
    );

    let h = crate::sync::get_db_height(connection, account).await?;
    let (ts, to) = crate::sync::get_tree_state(network, client, h.height).await?;
    let es = ts.to_edge(&SaplingHasher::default());
    let eo = to.to_edge(&OrchardHasher::default());
    let sapling_anchor = es.root(&SaplingHasher::default());
    let orchard_anchor = eo.root(&OrchardHasher::default());

    // generate a new change address if we need a transparent address
    let tkeys = select_account_transparent(connection, account, dindex).await?;
    let change_address = if change_pool == 0 && tkeys.xvk.is_some() {
        generate_next_change_address(network, connection, account)
            .await?
            .unwrap()
    } else {
        let change_scope = if use_internal { 1 } else { 0 };
        get_account_full_address(network, connection, account, change_scope, hw).await?
    };

    let mut outputs: Vec<RecipientState> = single
        .iter()
        .chain(double.iter())
        .cloned()
        .collect();

    // Add per-asset orchard change outputs
    for (asset_key, (used, outs)) in orchard_per_asset.iter() {
        let asset_change = used.saturating_sub(*outs);
        // For ZEC, also subtract fee (fees are ZEC)
        let asset_change = if asset_key == &zec_key {
            asset_change.saturating_sub(fee)
        } else {
            asset_change
        };
        if asset_change > 0 {
            outputs.push(RecipientState {
                recipient: Recipient {
                    address: change_address.clone(),
                    amount: asset_change,
                    asset_base: asset_key.clone(),
                    ..Recipient::default()
                },
                remaining: 0,
                pool_mask: PoolMask::from_pool(2), // orchard change
                asset_base: asset_key.clone(),
            });
        }
    }
    // Non-orchard change (transparent, sapling)
    let non_orchard_change = non_orchard_input
        .saturating_sub(
            single.iter().chain(double.iter())
                .filter(|r| r.pool_mask.to_best_pool().unwrap_or(2) != 2)
                .map(|r| r.recipient.amount).sum::<u64>()
        );
    if non_orchard_change > 0 && change_pool != 2 {
        outputs.push(RecipientState {
            recipient: Recipient {
                address: change_address,
                amount: non_orchard_change,
                ..Recipient::default()
            },
            remaining: 0,
            pool_mask: PoolMask::from_pool(change_pool),
            asset_base: zec_key.clone(),
        });
    }

    info!("Initializing Builder");

    let current_height = client.latest_height().await?;
    let target_height = current_height +
        // on regtest, add ZIP212_GRACE_PERIOD to make sure
        // ZIP-212 is enforced
        if network.network_type() == NetworkType::Regtest {
            ZIP212_GRACE_PERIOD
        } else { 0 };

    let mut builder = Builder::new(
        network,
        BlockHeight::from_u32(target_height),
        BuildConfig::Standard {
            sapling_anchor: sapling_crypto::Anchor::from_bytes(sapling_anchor).into_option(),
            orchard_anchor: orchard::Anchor::from_bytes(orchard_anchor).into_option(),
        },
    );

    let es = es.to_auth_path(&SaplingHasher::default());
    let eo = eo.to_auth_path(&OrchardHasher::default());

    let ers = empty_roots(&SaplingHasher::default());
    let ero = empty_roots(&OrchardHasher::default());

    let svk = get_sapling_vk(connection, account).await?;
    let ovk = get_orchard_vk(connection, account).await?;

    let mut tsk_dindex = vec![];
    let mut s_scope = vec![];

    event!(Level::INFO, "Adding Inputs");

    let ssk = get_sapling_sk(&mut *connection, account).await?;
    let osk = get_orchard_sk(&mut *connection, account).await?;

    // Sync orchard notes from orchard_buckets back to input_pools[2] for the
    // builder input loop, since fill_single_receivers modified the buckets copy.
    // ZEC notes go first so the builder processes ZEC spends before ZSA spends.
    input_pools[2].clear();
    if let Some(zec_notes) = orchard_buckets.get(&zec_key) {
        input_pools[2].extend(zec_notes.clone());
    }
    for (asset_key, notes) in orchard_buckets.iter() {
        if asset_key != &zec_key {
            input_pools[2].extend(notes.clone());
        }
    }

    let mut n_spends: [usize; 3] = [0, 0, 0];
    let mut inputs = vec![];
    for pool in input_pools.iter() {
        for inp in pool.iter() {
            if inp.is_used() {
                let InputNote {
                    id, amount, pool, ..
                } = inp;
                n_spends[*pool as usize] += 1;
                inputs.push(TxPlanIn {
                    amount: Some(*amount),
                    pool: *pool,
                });
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

                        let nf: Vec<u8> = row.get(0);
                        let pk: Vec<u8> = row.get(1);
                        let sk: Option<Vec<u8>> = row.get(2);
                        let scope: u32 = row.get(3);
                        let dindex: u32 = row.get(4);
                        let taddress: String = row.get(5);
                        let uncompressed: bool = row.get(6);

                        if sk.is_none() {
                            can_sign = false;
                        }

                        let pubkey = PublicKey::from_slice(&pk).unwrap();
                        let mut hash = [0u8; 32];
                        hash.copy_from_slice(&nf[0..32]);
                        let n = u32::from_le_bytes(nf[32..36].try_into().unwrap());
                        let utxo = OutPoint::new(hash, n);
                        let pk_bytes = if uncompressed {
                            pubkey.serialize_uncompressed().to_vec()
                        } else {
                            pubkey.serialize().to_vec()
                        };
                        let pkh: [u8; 20] = Ripemd160::digest(Sha256::digest(&pk_bytes)).into();
                        let addr = TransparentAddress::PublicKeyHash(pkh);
                        let coin =
                            TxOut::new(Zatoshis::from_u64(*amount).unwrap(), addr.script().into());

                        info!("Adding transparent input {}", hex::encode(utxo.hash()));
                        builder.add_transparent_input(
                            TransparentInputInfo::from_parts(utxo, coin, SpendInfo::P2pkh { pubkey, uncompressed })
                                .map_err(|e: zcash_transparent::builder::Error| anyhow!(e))?,
                        );
                        tsk_dindex.push((pubkey, scope, dindex, taddress, uncompressed));
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

                        info!(
                            "Adding sapling input {}",
                            hex::encode(note.cmu().to_bytes())
                        );
                        let dfvk = svk.as_ref().unwrap();
                        let fvk = sapling_dfvk_to_fvk(scope, dfvk);
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
                        )
                        .await?;

                        if osk.is_none() {
                            can_sign = false;
                        }

                        let is_zec = note.asset().is_zatoshi().into();
                        let asset_label = if is_zec {
                            "ZEC".to_string()
                        } else {
                            hex::encode(note.asset().to_bytes())
                        };
                        info!(
                            "Adding orchard input {} value={} asset={}",
                            hex::encode(
                                ExtractedNoteCommitment::from(note.commitment()).to_bytes()
                            ),
                            to_zec(note.value().inner()),
                            asset_label
                        );
                        builder.add_orchard_spend::<Infallible>(
                            ovk.clone().unwrap(),
                            note,
                            merkle_path,
                        )?;
                    }
                    _ => {}
                }

                let (nf,): (Vec<u8>,) =
                    sqlx::query_as("SELECT nullifier FROM notes WHERE id_note = ?")
                        .bind(id)
                        .fetch_one(&mut *connection)
                        .await?;
                debug!(
                    "id: {id}, pool: {pool}, nullifier: {}, amount: {}",
                    hex::encode(nf),
                    to_zec(*amount)
                );
            }
        }
    }

    event!(Level::INFO, "Adding Outputs");
    let mut n_outputs: [usize; 3] = [0, 0, 0];
    let mut outs = vec![];
    for r in outputs.iter() {
        let RecipientState {
            recipient,
            remaining,
            pool_mask,
            ..
        } = r;
        assert_eq!(*remaining, 0);
        assert!(pool_mask.single_pool());

        outs.push(TxPlanOut {
            pool: pool_mask.to_best_pool().unwrap(),
            amount: recipient.amount,
            address: recipient.address.clone(),
        });

        let pool = pool_mask.to_best_pool().unwrap();
        let value = Zatoshis::from_u64(recipient.amount)?;
        let memo = encode_memo(recipient)?.unwrap_or(MemoBytes::empty());

        n_outputs[pool as usize] += 1;
        match pool {
            0 => {
                // Don't add transparent outputs that have no value
                // because it is considered dust by the zcashd nodes
                if value != Zatoshis::ZERO {
                    let to = get_transparent_address(network, &recipient.address)?;
                    info!(
                        "Adding transparent output {} {}",
                        &recipient.address,
                        to_zec(value.into())
                    );
                    builder
                        .add_transparent_output(&to, value)
                        .map_err(|e: zcash_transparent::builder::Error| anyhow!(e))?;
                }
            }
            1 => {
                let to = get_sapling_address(network, &recipient.address)?;
                info!(
                    "Adding sapling output {} {}",
                    &recipient.address,
                    to_zec(value.into())
                );
                builder.add_sapling_output::<Infallible>(
                    svk.as_ref().map(|svk| svk.to_ovk(Scope::External)),
                    to,
                    value,
                    memo,
                )?;
            }
            2 => {
                let to = get_orchard_address(network, &recipient.address)?;
                let asset_base = if r.asset_base == zec_key {
                    AssetBase::zatoshi()
                } else {
                    AssetBase::from_bytes(&r.asset_base.clone().try_into().unwrap()).unwrap()
                };
                info!(
                    "Adding orchard output {} {} asset={}",
                    &recipient.address,
                    to_zec(value.into()),
                    hex::encode(&r.asset_base)
                );
                builder.add_orchard_output::<Infallible>(
                    ovk.as_ref().map(|ovk| ovk.to_ovk(Scope::External)),
                    to,
                    value,
                    asset_base,
                    memo,
                )?;
            }
            _ => {}
        }
    }

    event!(Level::INFO, "Building PCZT parts");
    let r = builder.build_for_pczt(OsRng, &FeeRule::standard(), |_| false)?;
    let sapling_meta = &r.sapling_meta;
    let orchard_meta = &r.orchard_meta;
    info!("PCZT parts built");

    let pczt = Creator::build_from_parts(r.pczt_parts).unwrap();
    info!("PCZT created");

    let updater = Updater::new(pczt);
    let updater = updater
        .update_transparent_with(|mut u| {
            for (i, (pubkey, scope, dindex, taddress, uncompressed)) in tsk_dindex.into_iter().enumerate() {
                u.update_input_with(i, |mut u| {
                    let derivation_path = vec![scope, dindex];
                    let path = Bip32Derivation::parse([0u8; 32], derivation_path).unwrap();
                    u.set_bip32_derivation(pubkey.serialize().to_vec(), path);
                    u.set_proprietary("scope".to_string(), scope.to_le_bytes().to_vec());
                    u.set_proprietary("dindex".to_string(), dindex.to_le_bytes().to_vec());
                    u.set_proprietary("address".to_string(), taddress.into_bytes());
                    u.set_proprietary("uncompressed".to_string(), vec![uncompressed as u8]);
                    // Set the hash160 preimage with the public key in the correct format
                    // This is needed for the signer to find the correct pubkey when verifying
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

            let mut c_output = 0;
            for o in outputs.iter() {
                let pool = o.pool_mask.to_best_pool().unwrap();
                if pool != 1 {
                    continue;
                }
                let bundle_index = sapling_meta.output_index(c_output).unwrap();
                u.update_output_with(bundle_index, |mut u| {
                    u.set_user_address(o.recipient.address.clone());
                    Ok(())
                })?;
                c_output += 1;
            }

            Ok(())
        })
        .unwrap();

    let updater = updater
        .update_orchard_with(|mut u| {
            let mut i = 0;
            for o in outputs.iter() {
                let pool = o.pool_mask.to_best_pool().unwrap();
                if pool != 2 {
                    continue;
                }
                let bundle_index = orchard_meta.output_action_index(i).unwrap();
                u.update_action_with(bundle_index, |mut u| {
                    u.set_output_user_address(o.recipient.address.clone());
                    Ok(())
                })?;
                i += 1;
            }

            Ok(())
        })
        .unwrap();

    let pczt = updater.finish();
    info!("PCZT metadata updated");

    let pczt = IoFinalizer::new(pczt).finalize_io().unwrap();
    info!("IO finalized");

    let pczt_package = PcztPackage {
        pczt: pczt.serialize(),
        n_spends,
        sapling_indices: (0..n_spends[1])
            .map(|n| sapling_meta.spend_index(n).unwrap())
            .collect(),
        orchard_indices: (0..n_spends[2])
            .map(|n| orchard_meta.spend_action_index(n).unwrap())
            .collect(),
        can_sign,
        can_broadcast: false,
        price,
        category,
    };

    Ok(pczt_package)
}

pub fn sapling_dfvk_to_fvk(scope: u32, dfvk: &DiversifiableFullViewingKey) -> FullViewingKey {
    let fvk = if scope == 0 {
        dfvk.fvk().clone()
    } else {
        dfvk.to_internal_fvk()
    };
    fvk
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
    pczt: &PcztPackage,
) -> Result<PcztPackage> {
    let span = span!(Level::INFO, "transaction");

    let PcztPackage {
        pczt,
        n_spends,
        sapling_indices,
        orchard_indices,
        price,
        category,
        ..
    } = pczt;
    let pczt = Pczt::parse(pczt).unwrap();

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
                    if scope == 0 {
                        u.set_proof_generation_key(pgk.clone().expect("proof_generation_key"))
                            .unwrap();
                    } else {
                        u.set_proof_generation_key(
                            internal_pgk.clone().expect("internal_proof_generation_key"),
                        )
                        .unwrap();
                    }

                    Ok(())
                })
                .unwrap();
            }
            Ok(())
        })
        .unwrap();
    let pczt = updater.finish();
    debug!("Updated");

    let mut signer = Signer::new(pczt.clone()).unwrap();
    let tbundle = pczt.transparent();
    let sbundle = pczt.sapling();
    for index in 0..n_spends[0] {
        debug!("signing transparent {index}");
        let inp = &tbundle.inputs()[index];
        let scope = u32::from_le_bytes(inp.proprietary()["scope"].clone().try_into().unwrap());
        let dindex = u32::from_le_bytes(inp.proprietary()["dindex"].clone().try_into().unwrap());
        // Check if "uncompressed" flag exists in proprietary, default to false (compressed)
        let uncompressed_flag = if let Some(val) = inp.proprietary().get("uncompressed") {
            if !val.is_empty() {
                val[0] != 0
            } else {
                info!("Invalid uncompressed flag length: {}, defaulting to compressed", val.len());
                false
            }
        } else {
            info!("No 'uncompressed' proprietary field found, defaulting to compressed");
            false
        };
        info!("Signing transparent input {}: scope={}, dindex={}, uncompressed={}", index, scope, dindex, uncompressed_flag);

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
        info!("Derived pubkey (compressed): hash={}, len={}", hex::encode(hash_compressed), derived_compressed.len());
        info!("Derived pubkey (uncompressed): hash={}, len={}", hex::encode(hash_uncompressed), 65);

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
        debug!("signing sapling {index}");
        let spend = &sbundle.spends()[*bundle_index];
        let scope = u32::from_le_bytes(spend.proprietary()["scope"].clone().try_into().unwrap());
        let ssk = ssk.as_ref().map(|ssk| {
            if scope == 0 {
                ssk.clone()
            } else {
                ssk.derive_internal()
            }
        });
        let Some(sk) = ssk.as_ref().map(|sk| &sk.expsk.ask) else {
            return Err(Error::NoSigningKey.into());
        };
        signer.sign_sapling(*bundle_index, sk).unwrap();
    }
    for (index, bundle_index) in orchard_indices.iter().enumerate() {
        debug!("signing orchard {index}");
        let Some(osak) = osak.as_ref() else {
            return Err(Error::NoSigningKey.into());
        };
        signer.sign_orchard(*bundle_index, osak).unwrap();
    }
    let pczt = signer.finish();

    span.in_scope(|| {
        info!("Adding Proofs to PCZT");
    });
    let sapling_prover: &LocalTxProver = &SAPLING_PROVER;

    let pczt = Prover::new(pczt)
        .create_sapling_proofs(sapling_prover, sapling_prover)
        .unwrap()
        .create_orchard_proof(&ORCHARD_PK)
        .unwrap()
        .finish();
    debug!("Proved");

    let pczt = SpendFinalizer::new(pczt).finalize_spends().unwrap();
    debug!("Spend Finalized");

    Ok(PcztPackage {
        pczt: pczt.serialize(),
        n_spends: *n_spends,
        sapling_indices: sapling_indices.clone(),
        orchard_indices: orchard_indices.clone(),
        can_sign: true,
        can_broadcast: true,
        price: *price,
        category: *category,
    })
}

pub async fn extract_transaction(package: &PcztPackage) -> Result<Vec<u8>> {
    let span = span!(Level::INFO, "transaction");
    span.in_scope(|| {
        info!("Extracting Tx");
    });

    let pczt = Pczt::parse(&package.pczt).unwrap();

    let sapling_prover: &LocalTxProver = &SAPLING_PROVER;
    let (svk, ovk) = sapling_prover.verifying_keys();
    let tx_extractor = TransactionExtractor::new(pczt).with_sapling(&svk, &ovk);
    let tx = tx_extractor.extract().unwrap();
    let mut tx_bytes = vec![];
    tx.write(&mut tx_bytes).unwrap();
    debug!("Tx Extracted");

    span.in_scope(|| {
        info!("Tx Ready - {} bytes", tx_bytes.len());
    });
    debug!("{}", hex::encode(&tx_bytes));

    Ok(tx_bytes)
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
    tracing::info!("{address}");
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

fn fill_single_receivers(
    input_pools: &mut [Vec<InputNote>],
    orchard_buckets: &mut HashMap<Vec<u8>, Vec<InputNote>>,
    recipients: &mut [RecipientState],
    fee_manager: &mut FeeManager,
    recipient_pays_fee: bool,
    fee_paid: &mut u64,
) -> Result<()> {
    for r in recipients.iter() {
        fee_manager.add_output(r.pool_mask.to_best_pool().unwrap());
    }

    let fill_order: [(u8, u8); 9] = [
        (2, 2),
        (1, 1), // O->O, S->S
        (2, 1),
        (1, 2), // O->S, S->O
        (0, 2),
        (0, 1), // T->O, T->S
        (2, 0),
        (1, 0), // O->T, S->T
        (0, 0), // T->T
    ];

    let zec_key = [0u8; 32].to_vec();

    for (src, dst) in fill_order {
        for r in recipients.iter_mut() {
            // skip if the recipient is not interested in this pool
            if r.pool_mask.intersect(&PoolMask::from_pool(dst)).is_empty() {
                continue;
            }

            if src == 2 {
                // Orchard: pay fees from ZEC bucket, recipient from asset bucket.
                let is_zec = r.asset_base == zec_key;

                // First pass: pay fees from ZEC orchard notes if not yet covered
                let fee_remaining = if recipient_pays_fee { 0 } else { fee_manager.fee() - *fee_paid };
                if fee_remaining > 0 {
                    if let Some(zec_bucket) = orchard_buckets.get_mut(&zec_key) {
                        for inp in zec_bucket.iter_mut() {
                            if inp.remaining == 0 || inp.amount < COST_PER_ACTION { continue; }
                            if inp.amount == inp.remaining { fee_manager.add_input(src); }
                            let fr = if recipient_pays_fee { 0 } else { fee_manager.fee() - *fee_paid };
                            if fr == 0 { break; }
                            let a = inp.remaining.min(fr);
                            *fee_paid += a;
                            inp.remaining -= a;
                        }
                    }
                }

                // Second pass: pay recipient from asset-specific bucket
                let bucket = if is_zec {
                    orchard_buckets.get_mut(&zec_key)
                } else {
                    orchard_buckets.get_mut(&r.asset_base)
                };
                if let Some(bucket) = bucket {
                    for inp in bucket.iter_mut() {
                        if inp.remaining == 0 || inp.amount < COST_PER_ACTION { continue; }

                        // Count the input for fee calculation (both ZEC and non-ZEC).
                        let is_first_touch = inp.amount == inp.remaining;
                        if is_first_touch {
                            fee_manager.add_input(src);
                        }

                        let fee_remaining = if recipient_pays_fee || !is_zec { 0 } else { fee_manager.fee() - *fee_paid };
                        let to_pay = r.remaining + fee_remaining;
                        if to_pay == 0 { break; }

                        let mut amount = inp.remaining.min(to_pay);
                        let a = amount.min(fee_remaining);
                        *fee_paid += a;
                        inp.remaining -= a;
                        amount -= a;
                        r.remaining -= amount;
                        inp.remaining -= amount;

                        debug!(
                            "Input id: {}, amount: {}, remaining: {}",
                            inp.id,
                            to_zec(inp.amount),
                            to_zec(inp.remaining)
                        );
                        debug!(
                            "Recipient id: {}, amount: {}, remaining: {}",
                            r.recipient.address,
                            to_zec(r.recipient.amount),
                            to_zec(r.remaining)
                        );
                    }
                }
            } else {
                // Transparent and sapling: existing behavior (all notes are ZEC)
                for inp in input_pools[src as usize].iter_mut() {
                    if inp.remaining == 0 || inp.amount < COST_PER_ACTION {
                        continue;
                    }

                    // if the recipient is not interested in this pool
                    if r.pool_mask.intersect(&PoolMask::from_pool(dst)).is_empty() {
                        continue;
                    }

                    // if the recipient pays the fees, we do not need to pay now
                    let fee_remaining = if recipient_pays_fee {
                        0
                    } else {
                        fee_manager.fee() - *fee_paid
                    };

                    if fee_remaining == 0 && r.remaining == 0 {
                        // nothing to pay anymore
                        break;
                    }

                    // first time we see this note, add it to the fee manager
                    if inp.amount == inp.remaining {
                        fee_manager.add_input(src)
                    }

                    // re-evaluate the fee after adding the input
                    // this is needed because the fee is based on the inputs
                    let fee_remaining = if recipient_pays_fee {
                        0
                    } else {
                        fee_manager.fee() - *fee_paid
                    };

                    // if the fee is not paid, we need to pay it on top of the output
                    let to_pay = r.remaining + fee_remaining;
                    // transfer the amount to the recipient
                    let mut amount = inp.remaining.min(to_pay);

                    // pay the fee first
                    let a = amount.min(fee_remaining);
                    *fee_paid += a;
                    inp.remaining -= a;
                    amount -= a;

                    // use the rest to pay the output
                    r.remaining -= amount;
                    inp.remaining -= amount;

                    debug!(
                        "Input id: {}, amount: {}, remaining: {}",
                        inp.id,
                        to_zec(inp.amount),
                        to_zec(inp.remaining)
                    );
                    debug!(
                        "Recipient id: {}, amount: {}, remaining: {}",
                        r.recipient.address,
                        to_zec(r.recipient.amount),
                        to_zec(r.remaining)
                    );
                }
            }
        }
    }

    Ok(())
}

pub async fn get_effective_src_pools(
    connection: &mut SqliteConnection,
    account: u32,
    src_pools: u8,
) -> Result<PoolMask> {
    let apm = get_account_pool_mask(connection, account).await?;
    let spm = PoolMask(src_pools);
    let src_pool_mask = apm.intersect(&spm);
    Ok(src_pool_mask)
}

pub fn get_change_pool(src_pool_mask: PoolMask, _dest_pool_mask: PoolMask) -> u8 {
    // pick the best pool from the source pools
    // because it can minimize the fees and reduce the amount going
    // through the turnstile
    src_pool_mask.to_best_pool().unwrap()
}

pub async fn get_account_pool_mask(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<PoolMask> {
    let (has_transparent,): (bool,) =
        sqlx::query_as("SELECT EXISTS(SELECT 1 FROM transparent_accounts WHERE account = ?)")
            .bind(account)
            .fetch_one(&mut *connection)
            .await?;
    let (has_sapling,): (bool,) =
        sqlx::query_as("SELECT EXISTS(SELECT 1 FROM sapling_accounts WHERE account = ?)")
            .bind(account)
            .fetch_one(&mut *connection)
            .await?;
    let (has_orchard,): (bool,) =
        sqlx::query_as("SELECT EXISTS(SELECT 1 FROM orchard_accounts WHERE account = ?)")
            .bind(account)
            .fetch_one(&mut *connection)
            .await?;
    let account_pool_mask =
        PoolMask((has_transparent as u8) | (has_sapling as u8) << 1 | (has_orchard as u8) << 2);

    Ok(account_pool_mask)
}

async fn fetch_one_taddr_unspent_notes(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<Vec<InputNote>> {
    let notes = sqlx::query(
        "SELECT a.id_note, a.height, a.value, a.taddress
        FROM notes a
        LEFT JOIN spends b ON a.id_note = b.id_note
        WHERE b.id_note IS NULL AND a.account = ?
        AND locked = 0
        AND a.pool = 0
        AND a.value >= 5000
        ORDER BY taddress",
    )
    .bind(account)
    .map(|row: SqliteRow| {
        let id: u32 = row.get(0);
        let height: u32 = row.get(1);
        let value: u64 = row.get(2);
        let taddress: u32 = row.get(3);
        (
            taddress,
            InputNote {
                id,
                height,
                amount: value,
                remaining: value,
                pool: 0, // transparent pool
                id_asset: None,
                asset_base: [0u8; 32].to_vec(),
            },
        )
    })
    .fetch_all(connection)
    .await?;

    let transparent_notes: Vec<Vec<_>> = notes
        .into_iter()
        .chunk_by(|item| item.0) // group by the transparent address
        .into_iter()
        .map(|group| group.1.map(|n| n.1).collect()) // collect each group of notes and discard the address
        .collect(); // collect all groups into Vec<Vec<_>>

    if !transparent_notes.is_empty() {
        // pick a random group to shield
        let random_note = OsRng.next_u32() as usize % transparent_notes.len();
        let notes = &transparent_notes[random_note];
        return Ok(notes.clone());
    }

    Ok(vec![])
}

pub async fn fetch_unspent_notes_grouped_by_pool(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<Vec<InputNote>> {
    let unspent_notes = sqlx::query(
        "SELECT a.id_note, a.height, a.pool, a.value, a.id_asset,
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
        let asset_base: Vec<u8> = row.get(5);
        InputNote {
            id: id_note,
            height,
            amount: value as u64,
            remaining: value as u64,
            pool,
            id_asset: id_asset.map(|v| v as u32),
            asset_base,
        }
    })
    .fetch_all(connection)
    .await?;

    Ok(unspent_notes)
}

pub static SAPLING_PROVER: LazyLock<LocalTxProver> = LazyLock::new(LocalTxProver::bundled);
pub static ORCHARD_PK: LazyLock<ProvingKey> = LazyLock::new(ProvingKey::build::<OrchardZSA>);
