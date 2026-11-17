use crate::api::coin::Network;
use crate::api::mempool::{MempoolAmount, MempoolMsg, MempoolNote, MempoolTx};
use crate::keys::{OrchardViewingKeys, SaplingViewingKeys};
use anyhow::{Context as _, Result};
use itertools::Itertools;
use orchard::note_encryption::{IronwoodDomain, OrchardDomain};
use sapling_crypto::note_encryption::SaplingDomain;
use sqlx::SqliteConnection;
use sqlx::{sqlite::SqliteRow, Row};
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;
use zcash_keys::{address::UnifiedAddress, encoding::AddressCodec as _};
use zcash_note_encryption::try_note_decryption;
use zcash_primitives::transaction::{
    components::sapling::zip212_enforcement, Authorized, OrchardBundle, Transaction,
    TransactionData,
};
use zcash_protocol::memo::Memo;
use zcash_transparent::address::TransparentAddress;

#[cfg(feature = "flutter")]
use crate::frb_generated::StreamSink;
use crate::{Client, Sink};

#[cfg(feature = "flutter")]
pub async fn run_mempool(
    mempool_tx: StreamSink<MempoolMsg>,
    network: &Network,
    connection: &mut SqliteConnection,
    client: &mut Client,
    cancel_token: CancellationToken,
) -> Result<()> {
    run_mempool_impl(mempool_tx, network, connection, client, cancel_token).await
}

pub async fn run_mempool_impl<S: Sink<MempoolMsg> + Send + 'static>(
    mempool_tx: S,
    network: &Network,
    connection: &mut SqliteConnection,
    client: &mut Client,
    cancel_token: CancellationToken,
) -> Result<()> {
    'outer: loop {
        let transparent_accounts = sqlx::query(
            r#"SELECT a.id_account, a.name, ta.address FROM accounts a
            JOIN transparent_address_accounts ta
            ON a.id_account = ta.account"#,
        )
        .map(|row: SqliteRow| {
            let account: u32 = row.get(0);
            let name: String = row.get(1);
            let address: String = row.get(2);
            let address = TransparentAddress::decode(network, &address).unwrap();
            (account, name, address)
        })
        .fetch_all(&mut *connection)
        .await
        .context("transparent_accounts")?;

        // Full or incoming-only keys per account; an incoming key sees the
        // external scope only.
        let sapling_accounts = sqlx::query(
            r#"SELECT account, name, xvk, ivk FROM accounts a JOIN sapling_accounts s
            ON a.id_account = s.account"#,
        )
        .map(|row: SqliteRow| {
            let account: u32 = row.get(0);
            let name: String = row.get(1);
            let keys = SaplingViewingKeys::from_stored(account, row.get(2), row.get(3));
            keys.map(|keys| keys.map(|keys| (account, name, keys)))
        })
        .fetch_all(&mut *connection)
        .await
        .context("sapling_accounts")?
        .into_iter()
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        let orchard_accounts = sqlx::query(
            r#"SELECT account, name, xvk, ivk FROM accounts a JOIN orchard_accounts o
            ON a.id_account = o.account"#,
        )
        .map(|row: SqliteRow| {
            let account: u32 = row.get(0);
            let name: String = row.get(1);
            let keys = OrchardViewingKeys::from_stored(account, row.get(2), row.get(3));
            keys.map(|keys| keys.map(|keys| (account, name, keys)))
        })
        .fetch_all(&mut *connection)
        .await
        .context("orchard_accounts")?
        .into_iter()
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        let height = client.latest_height().await?;
        mempool_tx.send(MempoolMsg::BlockHeight(height)).await;

        let mut mempool_txs = client.mempool_stream(network).await?;

        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    break 'outer;
                }
                r = mempool_txs.next() => {
                    match r {
                        // A stream silent for the client's whole idle
                        // window is most likely a long wait for a block
                        // (testnet, regtest), not a fault: reopen it, as at
                        // a new block (#1654).
                        Some(Err(e)) if crate::net::idle::is_idle_timeout(&e) => {
                            tracing::info!("the mempool stream was idle; reopening it: {e:#}");
                            break;
                        }
                        Some(Err(e)) => {
                            return Err(e.context("the mempool stream failed"));
                        }
                        Some(Ok((_, tx, len))) => {
                            let txid = tx.txid();
                            let tx_hash = txid.to_string();

                            let tx_data = tx.into_data();

                            let notes = decode_raw_transaction(
                                network,
                                connection,
                                height,
                                &transparent_accounts,
                                &sapling_accounts,
                                &orchard_accounts,
                                &tx_data,
                            ).await?;
                            let amounts = notes
                                .iter()
                                .map(|n| ((n.account, n.name.clone()), n.value))
                                .into_group_map()
                                .into_iter()
                                .map(|((account, name), values)| MempoolAmount {
                                    account,
                                    name,
                                    value: values.iter().sum(),
                                })
                                .collect::<Vec<_>>();
                            mempool_tx.send(MempoolMsg::TxId(MempoolTx {
                                txid: tx_hash,
                                amounts,
                                notes,
                                size: len as u32,
                            })).await;
                        }
                        None => {
                            break;
                        }
                    }
                }

            }
        }
    }
    Ok(())
}

fn memo_bytes_to_text(bytes: &[u8]) -> Option<String> {
    Memo::from_bytes(bytes).ok().and_then(|m| match m {
        Memo::Text(t) => Some(t.to_string()),
        _ => None,
    })
}

pub async fn decode_raw_transaction(
    network: &Network,
    connection: &mut SqliteConnection,
    height: u32,
    tkeys: &[(u32, String, TransparentAddress)],
    zkeys: &[(u32, String, SaplingViewingKeys)],
    okeys: &[(u32, String, OrchardViewingKeys)],
    tx_data: &TransactionData<Authorized>,
) -> Result<Vec<MempoolNote>> {
    let mut notes = vec![];

    if let Some(tbundle) = tx_data.transparent_bundle() {
        for v in tbundle.vin.iter() {
            let mut nf = vec![];
            v.prevout().write(&mut nf)?;
            let spent_amount = sqlx::query(
                "SELECT a.id_account, a.name, n.value, n.pool, n.scope, n.diversifier, m.memo_text
                FROM notes n
                JOIN accounts a ON n.account = a.id_account
                LEFT JOIN memos m ON m.note = n.id_note
                WHERE n.nullifier = ?",
            )
            .bind(&nf)
            .map(|row: SqliteRow| MempoolNote {
                account: row.get(0),
                name: row.get(1),
                value: -(row.get::<i64, _>(2)),
                pool: row.get(3),
                scope: row.get::<Option<u8>, _>(4).unwrap_or(0),
                diversifier: row.get(5),
                diversifier_index: None,
                address: None,
                memo: row.get(6),
            })
            .fetch_all(&mut *connection)
            .await?;
            notes.extend(spent_amount);
        }
        for v in tbundle.vout.iter() {
            if let Some(vout_address) = v.recipient_address() {
                if let Some((account, name, _)) = tkeys.iter().find(|(_, _, a)| a == &vout_address)
                {
                    notes.push(MempoolNote {
                        account: *account,
                        name: name.clone(),
                        value: v.value().into_u64() as i64,
                        pool: 0,
                        scope: 0,
                        diversifier: None,
                        diversifier_index: None,
                        address: Some(vout_address.encode(network)),
                        memo: None,
                    });
                }
            }
        }
    }

    if let Some(sbundle) = tx_data.sapling_bundle() {
        for v in sbundle.shielded_spends().iter() {
            let nf = &v.nullifier().to_vec();
            let spent_amount = sqlx::query(
                "SELECT a.id_account, a.name, n.value, n.pool, n.scope, n.diversifier, m.memo_text
                FROM notes n
                JOIN accounts a ON n.account = a.id_account
                LEFT JOIN memos m ON m.note = n.id_note
                WHERE n.nullifier = ?",
            )
            .bind(nf)
            .map(|row: SqliteRow| MempoolNote {
                account: row.get(0),
                name: row.get(1),
                value: -(row.get::<i64, _>(2)),
                pool: row.get(3),
                scope: row.get::<Option<u8>, _>(4).unwrap_or(0),
                diversifier: row.get(5),
                diversifier_index: None,
                address: None,
                memo: row.get(6),
            })
            .fetch_all(&mut *connection)
            .await?;
            notes.extend(spent_amount);
        }
        let domain = SaplingDomain::new(zip212_enforcement(network, height.into()));
        for v in sbundle.shielded_outputs().iter() {
            for (account, name, keys) in zkeys.iter() {
                for (scope, pivk) in keys.prepared_ivks() {
                    if let Some((note, recipient, memo_bytes)) =
                        try_note_decryption(&domain, &pivk, v)
                    {
                        let diversifier = recipient.diversifier().0.to_vec();
                        let address = recipient.encode(network);
                        let diversifier_index = keys.diversifier_index(scope, &diversifier);
                        notes.push(MempoolNote {
                            account: *account,
                            name: name.clone(),
                            value: note.value().inner() as i64,
                            pool: 1,
                            scope,
                            diversifier: Some(diversifier),
                            diversifier_index,
                            address: Some(address),
                            memo: memo_bytes_to_text(&memo_bytes),
                        });
                        break;
                    }
                }
            }
        }
    }

    macro_rules! process_orchard_bundle {
        ($bundle:expr, $pool:expr, $domain:ident) => {{
            let bundle = $bundle;
            let pool: u8 = $pool;
            for v in bundle.actions().iter() {
                let nf = v.nullifier().to_bytes().to_vec();
                let spent_amount = sqlx::query(
                    "SELECT a.id_account, a.name, n.value, n.pool, n.scope, n.diversifier, m.memo_text
                    FROM notes n
                    JOIN accounts a ON n.account = a.id_account
                    LEFT JOIN memos m ON m.note = n.id_note
                    WHERE n.nullifier = ?",
                )
                .bind(&nf)
                .map(|row: SqliteRow| MempoolNote {
                    account: row.get(0),
                    name: row.get(1),
                    value: -(row.get::<i64, _>(2)),
                    pool: row.get(3),
                    scope: row.get::<Option<u8>, _>(4).unwrap_or(0),
                    diversifier: row.get(5),
                    diversifier_index: None,
                    address: None,
                    memo: row.get(6),
                })
                .fetch_all(&mut *connection)
                .await?;
                notes.extend(spent_amount);

                let domain = $domain::for_action(v);
                for (account, name, keys) in okeys.iter() {
                    for (scope, pivk) in keys.prepared_ivks() {
                        if let Some((note, recipient, memo_bytes)) =
                            try_note_decryption(&domain, &pivk, v)
                        {
                            let diversifier = recipient.diversifier().as_array().to_vec();
                            let ua =
                                UnifiedAddress::from_receivers(Some(recipient), None, None, None, None).unwrap();
                            let address = ua.encode(network);
                            let diversifier_index = keys.diversifier_index(scope, &diversifier);
                            notes.push(MempoolNote {
                                account: *account,
                                name: name.clone(),
                                value: note.value().inner() as i64,
                                pool,
                                scope,
                                diversifier: Some(diversifier),
                                diversifier_index,
                                address: Some(address),
                                memo: memo_bytes_to_text(&memo_bytes),
                            });
                            break;
                        }
                    }
                }
            }
        }};
    }

    if let Some(obundle) = tx_data.orchard_bundle() {
        match obundle {
            OrchardBundle::OrchardVanilla(b) => {
                process_orchard_bundle!(b, 2, OrchardDomain);
            }
            OrchardBundle::OrchardZSA(_b) => {
                // TODO: ZSA mempool processing
            }
        }
    }
    if let Some(iwbundle) = tx_data.ironwood_bundle() {
        process_orchard_bundle!(iwbundle, 3, IronwoodDomain);
    }
    Ok(notes)
}

pub async fn get_mempool_tx(
    network: &Network,
    client: &mut Client,
    tx_id: &str,
) -> Result<Transaction> {
    let mut tx_id = hex::decode(tx_id)?;
    tx_id.reverse();
    let (_, tx) = client.transaction(network, &tx_id).await?;
    Ok(tx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lwd::CompactBlock;
    use crate::net::LwdServer;
    use sqlx::Connection as _;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tokio::sync::mpsc;
    use tokio_stream::wrappers::ReceiverStream;
    use tonic::async_trait;

    type TxStream = ReceiverStream<Result<(u32, Transaction, usize)>>;

    /// A server whose first mempool stream ends on an idle timeout and
    /// whose later ones stay open. Counts the streams opened.
    struct IdleOnce {
        opened: Arc<AtomicUsize>,
        /// Keeps the later streams open.
        open: Vec<mpsc::Sender<Result<(u32, Transaction, usize)>>>,
    }

    #[async_trait]
    impl LwdServer for IdleOnce {
        type CompactBlockStream = ReceiverStream<Result<CompactBlock>>;
        type TransactionStream = TxStream;

        async fn latest_height(&mut self) -> Result<u32> {
            Ok(100)
        }
        async fn block(&mut self, _: &Network, _: u32) -> Result<CompactBlock> {
            unimplemented!()
        }
        async fn block_range(
            &mut self,
            _: &Network,
            _: u32,
            _: u32,
        ) -> Result<Self::CompactBlockStream> {
            unimplemented!()
        }
        async fn transaction(&mut self, _: &Network, _: &[u8]) -> Result<(u32, Transaction)> {
            unimplemented!()
        }
        async fn post_transaction(&mut self, _: u32, _: &[u8]) -> Result<String> {
            unimplemented!()
        }
        async fn taddress_txs(
            &mut self,
            _: &Network,
            _: &str,
            _: u32,
            _: u32,
        ) -> Result<Self::TransactionStream> {
            unimplemented!()
        }
        async fn mempool_stream(&mut self, _: &Network) -> Result<Self::TransactionStream> {
            let (tx, rx) = mpsc::channel(1);
            if self.opened.fetch_add(1, Ordering::SeqCst) == 0 {
                let idle = tonic::Status::deadline_exceeded("no data for 300s on GetMempoolStream");
                tx.send(Err(anyhow::Error::from(idle))).await.unwrap();
            } else {
                self.open.push(tx);
            }
            Ok(ReceiverStream::new(rx))
        }
        async fn tree_state(&mut self, _: u32) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
            unimplemented!()
        }
    }

    /// A mempool stream that times out for being idle is reopened rather
    /// than ending the monitor, since on testnet or regtest a block can
    /// take longer than the window (#1654).
    #[tokio::test]
    async fn an_idle_stream_is_reopened() -> Result<()> {
        let mut db = SqliteConnection::connect("sqlite::memory:").await?;
        crate::db::create_schema(&mut db).await?;
        let opened = Arc::new(AtomicUsize::new(0));
        let mut client: Client = Box::new(IdleOnce { opened: opened.clone(), open: vec![] });
        let (sink, _messages) = mpsc::channel::<MempoolMsg>(16);
        let cancel = CancellationToken::new();
        let monitor = {
            let cancel = cancel.clone();
            async move {
                run_mempool_impl(sink, &Network::Main, &mut db, &mut client, cancel).await
            }
        };
        let monitor = tokio::spawn(monitor);
        for _ in 0..100 {
            if opened.load(Ordering::SeqCst) >= 2 {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        assert!(!monitor.is_finished(), "the monitor ended on the idle timeout");
        assert_eq!(opened.load(Ordering::SeqCst), 2, "the stream was not reopened");
        cancel.cancel();
        monitor.await??;
        Ok(())
    }
}
