use std::{convert::Infallible, str::FromStr as _};

use anyhow::{anyhow, Result};

use bip32::PrivateKey;
use flutter_rust_bridge::frb;
use itertools::Itertools;
use orchard::{circuit::ProvingKey, keys::{Scope, SpendAuthorizingKey}, Address};
use pczt::roles::{creator::Creator, io_finalizer::IoFinalizer, prover::Prover, signer::Signer, spend_finalizer::SpendFinalizer, tx_extractor::{self, TransactionExtractor}, updater::{self, Updater}};
use rand_core::OsRng;
use ripemd::Ripemd160;
use sapling_crypto::{Note, PaymentAddress};
use secp256k1::SecretKey;
use sha2::{Digest as _, Sha256};
use zcash_keys::{address::UnifiedAddress, encoding::AddressCodec as _};
use zcash_primitives::{
    legacy::TransparentAddress,
    transaction::{builder::{BuildConfig, Builder}, fees::zip317::FeeRule},
};
use zcash_proofs::prover::LocalTxProver;
use zcash_protocol::{
    consensus::{BlockHeight, Network},
    memo::{Memo, MemoBytes},
    value::Zatoshis,
};
use zcash_transparent::bundle::{OutPoint, TxOut};

use crate::{
    account::{get_account_full_address, get_orchard_note, get_orchard_sk, get_orchard_vk, get_sapling_note, get_sapling_sk, get_sapling_vk},
    pay::{
        error::Error,
        fee::{FeeManager, COST_PER_ACTION},
        plan::{fetch_unspent_notes_grouped_by_pool, get_change_pool},
        pool::{PoolMask, ALL_POOLS},
        prepare::to_zec,
        InputNote, Recipient, RecipientState,
    },
    warp::hasher::{empty_roots, OrchardHasher, SaplingHasher},
};

#[frb]
pub async fn prepare(account: u32, sender_pay_fees: bool, src_pools: u8) -> Result<()> {
    let c = crate::get_coin!();
    let network = &c.network;
    let connection = c.get_pool();
    let mut client = c.client().await?;
    crate::pay::prepare::prepare(
        network,
        connection,
        &mut client,
        account,
        &[],
        sender_pay_fees,
        src_pools,
    )
    .await?;

    Ok(())
}

// #[frb]
pub async fn wip_plan(
    account: u32,
    src_pools: u8,
    recipients: &[Recipient],
    recipient_pays_fee: bool,
) -> Result<()> {
    let c = crate::get_coin!();
    let network = &c.network;
    let connection = c.get_pool();
    let effective_src_pools =
        crate::pay::plan::get_effective_src_pools(connection, account, src_pools).await?;

    let mut recipients = recipients.to_vec();
    let mut recipient_pools = PoolMask(0);
    for recipient in recipients.iter() {
        let pool = PoolMask::from_address(&recipient.address)?
            .intersect(&PoolMask(recipient.pools.unwrap_or(ALL_POOLS)));
        recipient_pools = recipient_pools.union(&pool);
    }
    println!("effective_src_pools: {:#b}", effective_src_pools.0);
    println!("recipient_pools: {:#b}", recipient_pools.0);
    let change_pool = get_change_pool(effective_src_pools, recipient_pools);
    println!("change_pool: {:#b}", change_pool);

    let mut fee_manager = FeeManager::default();
    fee_manager.add_output(change_pool);

    let recipient_states = recipients
        .iter()
        .map(|r| RecipientState::new(r.clone()).unwrap())
        .collect::<Vec<_>>();
    let mut input_pools = vec![vec![]; 3];
    let inputs = fetch_unspent_notes_grouped_by_pool(connection, account).await?;

    for (group, items) in inputs.into_iter().chunk_by(|inp| inp.pool).into_iter() {
        input_pools[group as usize].extend(items);
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

    let mut fee_paid = 0;
    fill_single_receivers(
        &mut input_pools,
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
    //
    // In the second pass, we constrain the receiver to be the pool
    // that we have the most balance in. This is because we hope
    // to minimize the amount that would have to go through the
    // turnstile.

    let balances = input_pools
        .iter()
        .map(|pool| pool.iter().map(|n| n.remaining).sum::<u64>())
        .collect::<Vec<_>>();

    let largest_shielded_pool = if balances[1] > balances[2] {
        PoolMask(2)
    } else {
        PoolMask(4)
    };

    for d in double.iter_mut() {
        d.pool_mask = largest_shielded_pool;
    }

    fill_single_receivers(
        &mut input_pools,
        &mut double,
        &mut fee_manager,
        recipient_pays_fee,
        &mut fee_paid,
    )?;

    let fee = fee_manager.fee();

    if recipient_pays_fee {
        if recipients.len() > 1 {
            // if there are multiple recipients, we error because we
            // do not know which recipient should pay the fee
            return Err(Error::TooManyRecipients.into());
        } else {
            let recipient = recipients.first_mut().unwrap();
            if recipient.amount < fee {
                // if the recipient does not have enough balance to pay
                // the fee, we need to pay it from the input pools
                return Err(Error::RecipientNotEnoughAmount.into());
            }
            recipient.amount -= fee;
            fee_paid += fee;
        }
    }

    if single.iter().any(|r| r.remaining > 0)
        || double.iter().any(|r| r.remaining > 0)
        || fee > fee_paid
    {
        return Err(Error::NotEnoughFunds.into());
    }

    let total_input = input_pools
        .iter()
        .map(|pool| {
            pool.iter()
                .map(|n| if n.is_used() { n.amount } else { 0 })
                .sum::<u64>()
        })
        .sum::<u64>();
    let total_output = recipients.iter().map(|r| r.amount).sum::<u64>();

    let change = total_input - total_output - fee;

    for o in single.iter().chain(double.iter()) {
        let RecipientState {
            recipient,
            remaining,
            pool_mask,
        } = o;
        assert_eq!(*remaining, 0);
        println!(
            "address: {}, pool: {}, amount: {}",
            recipient.address,
            pool_mask.to_best_pool().unwrap(),
            to_zec(recipient.amount)
        );
    }

    println!(
        "change: {}, pool: {change_pool}, fee: {}",
        to_zec(change),
        to_zec(fee)
    );

    let height = crate::sync::get_db_height(connection, account).await?;
    let mut client = c.client().await?;
    let (ts, to) = crate::sync::get_tree_state(&c.network, &mut client, height).await?;
    let es = ts.to_edge(&SaplingHasher::default());
    let eo = to.to_edge(&OrchardHasher::default());
    let sapling_anchor = es.root(&SaplingHasher::default());
    let orchard_anchor = eo.root(&OrchardHasher::default());

    let change_address = get_account_full_address(
        network,
        connection,
        account,
    ).await?;

    let change_recipient = RecipientState {
        recipient: Recipient {
            address: change_address,
            amount: change,
            ..Recipient::default()
        },
        remaining: 0,
        pool_mask: PoolMask::from_pool(change_pool),
    };

    let outputs = single.iter().chain(double.iter()).chain(std::iter::once(&change_recipient));

    let mut builder = Builder::new(
        &c.network,
        BlockHeight::from_u32(height),
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

    let mut tsk = vec![];

    let mut n_spends: [usize; 3] = [0, 0, 0];
    for pool in input_pools.iter() {
        for inp in pool.iter() {
            if inp.is_used() {
                let InputNote {
                    id, amount, pool, ..
                } = inp;
                n_spends[*pool as usize] += 1;
                match pool {
                    0 => {
                        let (nf, sk): (Vec<u8>, Vec<u8>) = sqlx::query_as(
                            "SELECT nullifier, t.sk FROM notes
                            JOIN transparent_address_accounts t ON notes.taddress = t.id_taddress
                            WHERE id_note = ?",
                        )
                        .bind(*id)
                        .fetch_one(connection)
                        .await?;

                        let sk = SecretKey::from_bytes(&sk.try_into().unwrap()).unwrap();
                        let pubkey = sk.public_key(&secp256k1::Secp256k1::new());
                        let mut hash = [0u8; 32];
                        hash.copy_from_slice(&nf[0..32]);
                        let n = u32::from_le_bytes(nf[32..36].try_into().unwrap());
                        let utxo = OutPoint::new(hash, n);
                        let pkh: [u8; 20] =
                            Ripemd160::digest(&Sha256::digest(&pubkey.serialize())).into();
                        let addr = TransparentAddress::PublicKeyHash(pkh.clone());
                        let coin = TxOut {
                            value: Zatoshis::from_u64(*amount).unwrap(),
                            script_pubkey: addr.script(),
                        };

                        builder
                            .add_transparent_input(pubkey, utxo, coin)
                            .map_err(|e| anyhow!(e))?;
                        tsk.push(sk);
                    }
                    1 => {
                        let (note, merkle_path) =
                            get_sapling_note(connection, *id, height, &svk, &es, &ers).await?;

                        builder.add_sapling_spend::<Infallible>(svk.clone(), note, merkle_path)?;
                    }
                    2 => {
                        let (note, merkle_path) =
                            get_orchard_note(connection, *id, height, &ovk, &eo, &ero).await?;

                        builder.add_orchard_spend::<Infallible>(ovk.clone(), note, merkle_path)?;
                    }
                    _ => {}
                }

                let (nf,): (Vec<u8>,) =
                    sqlx::query_as("SELECT nullifier FROM notes WHERE id_note = ?")
                        .bind(id)
                        .fetch_one(connection)
                        .await?;
                println!(
                    "id: {id}, pool: {pool}, nullifier: {}, amount: {}",
                    hex::encode(nf),
                    to_zec(*amount)
                );
            }
        }
    }

    for r in outputs {
        let RecipientState {
            recipient,
            remaining,
            pool_mask,
        } = r;
        assert_eq!(*remaining, 0);
        assert!(pool_mask.single_pool());

        let pool = pool_mask.to_best_pool().unwrap();
        let value = Zatoshis::from_u64(recipient.amount)?;
        let text_memo = recipient
            .user_memo
            .as_ref()
            .map(|s| Memo::from_str(&s))
            .transpose()?
            .map(MemoBytes::from);
        let byte_memo = recipient
            .memo_bytes
            .as_ref()
            .map(|mb| MemoBytes::from_bytes(&mb))
            .transpose()?;
        let memo = text_memo.or(byte_memo).unwrap_or(MemoBytes::empty());

        match pool {
            0 => {
                let to = get_transparent_address(network, &recipient.address)?;
                builder
                    .add_transparent_output(&to, value)
                    .map_err(|e| anyhow!(e))?;

            }
            1 => {
                let to = get_sapling_address(network, &recipient.address)?;
                builder.add_sapling_output::<Infallible>(Some(svk.ovk.clone()), to, value, memo)?;
            }
            2 => {
                let to = get_orchard_address(network, &recipient.address)?;
                builder.add_orchard_output::<Infallible>(
                    Some(ovk.to_ovk(Scope::External)),
                    to,
                    value.into_u64(),
                    memo,
                )?;
            }
            _ => {}
        }
    }

    println!("Building");
    let r = builder.build_for_pczt(OsRng, &FeeRule::standard())?;
    let sapling_meta = &r.sapling_meta;
    let orchard_meta = &r.orchard_meta;

    println!("Prepared");

    let sapling_prover: &LocalTxProver = &SAPLING_PROVER;

    let pczt = Creator::build_from_parts(r.pczt_parts).unwrap();
    println!("Created");

    let pczt = IoFinalizer::new(pczt).finalize_io().unwrap();
    println!("IO Finalized");

    let ssk = get_sapling_sk(connection, account).await?;
    let osk = get_orchard_sk(connection, account).await?;
    let osak = SpendAuthorizingKey::from(&osk);

    let updater = Updater::new(pczt);
    let pgk = ssk.expsk.proof_generation_key();
    let updater = updater.update_sapling_with(|mut u| {
        for i in 0..n_spends[1] {
            let bundle_index = sapling_meta.spend_index(i).unwrap();
            u.update_spend_with(bundle_index, |mut u| {
                u.set_proof_generation_key(pgk.clone()).unwrap();
                Ok(())
            }).unwrap();
        }
        Ok(())
    }).unwrap();
    let pczt = updater.finish();
    println!("Updated");

    let pczt = Prover::new(pczt)
        .create_sapling_proofs(sapling_prover, sapling_prover)
        .unwrap()
        .create_orchard_proof(&ORCHARD_PK)
        .unwrap()
        .finish();
    println!("Proved");

    let mut signer = Signer::new(pczt).unwrap();
    for index in 0..n_spends[0] {
        println!("signing transparent {index}");
        signer.sign_transparent(index, &tsk[index]).unwrap();
    }
    for index in 0..n_spends[1] {
        println!("signing sapling {index}");
        let bundle_index = sapling_meta.spend_index(index).unwrap();
        signer.sign_sapling(bundle_index, &ssk.expsk.ask).unwrap();
    }
    for index in 0..n_spends[2] {
        println!("signing orchard {index}");
        let bundle_index = orchard_meta.spend_action_index(index).unwrap();
        signer.sign_orchard(bundle_index, &osak).unwrap();
    }
    let pczt = signer.finish();
    println!("Signed");

    let pczt = SpendFinalizer::new(pczt).finalize_spends().unwrap();
    println!("Spend Finalized");

    let (svk, ovk) = sapling_prover.verifying_keys();
    let tx_extractor = TransactionExtractor::new(pczt)
        .with_sapling(&svk, &ovk);
    let tx = tx_extractor.extract().unwrap();
    let mut tx_bytes = vec![];
    tx.write(&mut tx_bytes).unwrap();
    println!("Tx Extracted");

    println!("{}", hex::encode(&tx_bytes));

    Ok(())
}

fn get_transparent_address(network: &Network, address: &str) -> Result<TransparentAddress> {
    if let Ok(addr) = TransparentAddress::decode(network, address) {
        return Ok(addr);
    }
    if let Ok(addr) = UnifiedAddress::decode(network, address) {
        let addr = addr.transparent().unwrap().clone();
        return Ok(addr);
    } else {
        anyhow::bail!("Invalid transparent address: {address}");
    }
}

fn get_sapling_address(network: &Network, address: &str) -> Result<PaymentAddress> {
    if let Ok(addr) = PaymentAddress::decode(network, address) {
        return Ok(addr);
    }
    if let Ok(addr) = UnifiedAddress::decode(network, address) {
        let addr = addr.sapling().unwrap().clone();
        return Ok(addr);
    } else {
        anyhow::bail!("Invalid sapling address: {address}");
    }
}

fn get_orchard_address(network: &Network, address: &str) -> Result<Address> {
    if let Ok(addr) = UnifiedAddress::decode(network, address) {
        let addr = addr.orchard().unwrap().clone();
        return Ok(addr);
    } else {
        anyhow::bail!("Invalid orchard address: {address}");
    }
}

fn fill_single_receivers(
    input_pools: &mut Vec<Vec<InputNote>>,
    recipients: &mut [RecipientState],
    fee_manager: &mut FeeManager,
    recipient_pays_fee: bool,
    fee_paid: &mut u64,
) -> Result<()> {
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

    for (src, dst) in fill_order {
        for r in recipients.iter_mut() {
            if r.remaining == 0 {
                continue;
            }
            for inp in input_pools[src as usize].iter_mut() {
                if inp.remaining == 0 || inp.amount < COST_PER_ACTION {
                    continue;
                }
                // skip if the recipient is not interested in this pool
                if r.pool_mask.intersect(&PoolMask::from_pool(dst)).is_empty() {
                    continue;
                }
                // first time we see this note, add it to the fee manager
                if inp.amount == inp.remaining {
                    fee_manager.add_input(src)
                }

                // if the recipient pays the fees, we do not need to pay now
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
            }
        }
    }

    Ok(())
}

lazy_static::lazy_static! {
    static ref SAPLING_PROVER: LocalTxProver = LocalTxProver::bundled();
    static ref ORCHARD_PK: ProvingKey = ProvingKey::build();
}
