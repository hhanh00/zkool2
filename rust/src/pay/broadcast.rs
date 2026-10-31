//! What happens after a transaction is signed, in one place: post it, lock
//! the notes it spends, and remember the price and category the wallet
//! recorded for it. Every send path goes through here, whether it holds a
//! signed PCZT package or the raw transaction bytes.
//!
//! The two halves are separate: [`post`] is the network, [`record_sent`]
//! the chain-log writes that follow an acceptance. A caller under the sync
//! session (the protocol driver, mid-pass) does both in one go with
//! [`broadcast_under_session`]; the app and the server post, then queue the
//! writes through the sync driver, so a send never waits behind a sync and
//! never writes beside one.

use anyhow::{anyhow, Result};
use sqlx::SqliteConnection;
use tracing::{info, span, Level};
use zcash_primitives::transaction::{OrchardBundle, Transaction};
use zcash_protocol::consensus::BranchId;

use crate::api::pay::PcztPackage;
use crate::Client;

/// Wallet-side metadata to attach to the transaction once it is mined.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pending {
    pub price: Option<f64>,
    pub category: Option<u32>,
}

/// The package's price and category, to carry as pending metadata.
pub fn pending_of(package: &PcztPackage) -> Pending {
    Pending {
        price: package.price,
        category: package.category,
    }
}

/// Extract a signed package and broadcast it, carrying the package's price
/// and category as the pending metadata. For a caller holding the sync
/// session.
pub async fn broadcast_package_under_session(
    conn: &mut SqliteConnection,
    client: &mut Client,
    account: u32,
    package: &PcztPackage,
) -> Result<String> {
    let tx_bytes = super::plan::extract_transaction(package).await?;
    broadcast_under_session(conn, client, account, &tx_bytes, Some(pending_of(package))).await
}

/// What the server answered to an accepted transaction.
#[derive(Clone, Debug)]
pub struct Sent {
    /// The chain height the transaction was posted at.
    pub height: u32,
    pub txid: Vec<u8>,
    /// The server's answer, the txid in hex.
    pub response: String,
}

/// Post a serialized transaction; a rejection is an error carrying the
/// server's message. Network only: the chain-log writes that follow are
/// [`record_sent`].
pub async fn post(client: &mut Client, tx_bytes: &[u8]) -> Result<Sent> {
    let height = client.latest_height().await?;
    let span = span!(Level::INFO, "transaction");
    let response = client.post_transaction(height, tx_bytes).await?;
    // The server answers with the txid, or with its rejection message.
    let txid = hex::decode(&response).map_err(|_| anyhow!(response.clone()))?;
    span.in_scope(|| {
        info!("TXID: {}", response);
    });
    Ok(Sent {
        height,
        txid,
        response,
    })
}

/// After an acceptance: lock the notes the transaction spends, and record
/// the pending metadata if any. Called under the sync session, by the
/// driver or by a caller that holds it.
pub async fn record_sent(
    conn: &mut SqliteConnection,
    account: u32,
    sent: &Sent,
    tx_bytes: &[u8],
    pending: Option<Pending>,
) -> Result<()> {
    lock_spent_notes(conn, account, tx_bytes).await?;
    if let Some(pending) = pending {
        crate::db::store_pending_tx(
            conn,
            account,
            sent.height,
            &sent.txid,
            pending.price,
            pending.category,
        )
        .await?;
    }
    Ok(())
}

/// Post a serialized transaction. On acceptance, lock the notes it spends so
/// a later plan cannot select them before the spend is mined, record the
/// pending metadata when given, and return the txid. A rejection is an
/// error and leaves the wallet untouched.
pub async fn broadcast_under_session(
    conn: &mut SqliteConnection,
    client: &mut Client,
    account: u32,
    tx_bytes: &[u8],
    pending: Option<Pending>,
) -> Result<String> {
    let sent = post(client, tx_bytes).await?;
    record_sent(conn, account, &sent, tx_bytes, pending).await?;
    Ok(sent.response)
}

/// After a successful broadcast, mark the notes the transaction spends as
/// `locked` so note selection will not pick them again until the spend is
/// mined and synced (which clears the flag — see the chain log).
///
/// This closes the window where a second transaction is built from a note
/// that an earlier, still-unmined broadcast already spent — the "duplicate
/// nullifier" the node would otherwise reject (seen in rapid-fire DKG
/// funding rounds).
///
/// Best-effort: a transaction we cannot parse simply locks nothing. Nullifiers
/// are matched against `notes.nullifier` exactly as
/// [`crate::mempool::decode_raw_transaction`] does, which already covers every
/// pool including ironwood.
pub async fn lock_spent_notes(
    conn: &mut SqliteConnection,
    account: u32,
    tx_bytes: &[u8],
) -> Result<()> {
    let Some(nullifiers) = spent_nullifiers(tx_bytes)? else {
        info!("lock_spent_notes: could not parse transaction; locking nothing");
        return Ok(());
    };
    crate::sync::chain_log::lock_notes(conn, account, &nullifiers).await
}

/// The inverse of [`lock_spent_notes`], for a transaction the network will
/// not mine any more: its inputs are spendable again.
pub async fn unlock_spent_notes(
    conn: &mut SqliteConnection,
    account: u32,
    tx_bytes: &[u8],
) -> Result<()> {
    let Some(nullifiers) = spent_nullifiers(tx_bytes)? else {
        info!("unlock_spent_notes: could not parse transaction; unlocking nothing");
        return Ok(());
    };
    crate::sync::chain_log::unlock_notes(conn, account, &nullifiers).await
}

/// A serialized transaction, under whichever network upgrade it parses.
pub fn parse_transaction(tx_bytes: &[u8]) -> Option<Transaction> {
    [
        BranchId::Nu6_3,
        BranchId::Nu6_2,
        BranchId::Nu6,
        BranchId::Nu5,
    ]
    .into_iter()
    .find_map(|branch| Transaction::read(&mut &tx_bytes[..], branch).ok())
}

/// The input keys of a transaction, as the `nullifier` column stores them;
/// `None` when the bytes do not parse.
fn spent_nullifiers(tx_bytes: &[u8]) -> Result<Option<Vec<Vec<u8>>>> {
    let Some(tx) = parse_transaction(tx_bytes) else {
        return Ok(None);
    };
    let tx_data = tx.into_data();

    // Every input's key (transparent outpoint or shielded nullifier) is stored
    // in the `nullifier` column, so collect them all and lock the matches.
    let mut nullifiers: Vec<Vec<u8>> = vec![];
    if let Some(tbundle) = tx_data.transparent_bundle() {
        for v in tbundle.vin.iter() {
            let mut nf = vec![];
            v.prevout().write(&mut nf)?;
            nullifiers.push(nf);
        }
    }
    if let Some(sbundle) = tx_data.sapling_bundle() {
        for v in sbundle.shielded_spends().iter() {
            nullifiers.push(v.nullifier().to_vec());
        }
    }
    if let Some(obundle) = tx_data.orchard_bundle() {
        match obundle {
            OrchardBundle::OrchardVanilla(b) => {
                for v in b.actions().iter() {
                    nullifiers.push(v.nullifier().to_bytes().to_vec());
                }
            }
            OrchardBundle::OrchardZSA(b) => {
                for v in b.actions().iter() {
                    nullifiers.push(v.nullifier().to_bytes().to_vec());
                }
            }
        }
    }
    if let Some(iwbundle) = tx_data.ironwood_bundle() {
        for v in iwbundle.actions().iter() {
            nullifiers.push(v.nullifier().to_bytes().to_vec());
        }
    }
    Ok(Some(nullifiers))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::coin::Network;
    use crate::db::create_schema;
    use crate::lwd::CompactBlock;
    use crate::net::LwdServer;
    use crate::sync::chain_log::{self, NoteRow, TransactionRow};
    use sqlx::Connection;
    use tokio_stream::wrappers::ReceiverStream;
    use tonic::async_trait;
    use zcash_primitives::transaction::{Authorized, TransactionData, TxVersion};
    use zcash_protocol::consensus::BlockHeight;
    use zcash_protocol::value::Zatoshis;
    use zcash_transparent::bundle::{Bundle, OutPoint, TxIn};

    /// A lightwalletd that answers every post with a fixed response.
    struct Posts(String, Vec<Vec<u8>>);

    #[async_trait]
    impl LwdServer for Posts {
        type CompactBlockStream = ReceiverStream<Result<CompactBlock>>;
        type TransactionStream = ReceiverStream<Result<(u32, Transaction, usize)>>;

        async fn latest_height(&mut self) -> Result<u32> {
            Ok(500)
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
        async fn post_transaction(&mut self, _: u32, tx: &[u8]) -> Result<String> {
            self.1.push(tx.to_vec());
            Ok(self.0.clone())
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
            unimplemented!()
        }
        async fn tree_state(&mut self, _: u32) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
            unimplemented!()
        }
    }

    const TXID: &str = "0102030405060708091011121314151617181920212223242526272829303132";

    /// A v5 transaction spending one transparent outpoint and nothing else.
    #[allow(deprecated)] // TxIn has no constructor for a spend
    fn spend_of(outpoint: OutPoint) -> Vec<u8> {
        let bundle = Bundle {
            vin: vec![TxIn {
                prevout: outpoint,
                script_sig: Default::default(),
                sequence: 0xffff_ffff,
            }],
            vout: vec![],
            authorization: zcash_transparent::bundle::Authorized,
        };
        let tx = TransactionData::<Authorized>::from_parts(
            TxVersion::V5,
            BranchId::Nu5,
            0,
            BlockHeight::from_u32(0),
            Zatoshis::ZERO,
            Some(bundle),
            None,
            None,
            None,
        )
        .freeze()
        .unwrap();
        let mut bytes = vec![];
        tx.write(&mut bytes).unwrap();
        bytes
    }

    async fn wallet_with_note(outpoint: &OutPoint) -> Result<SqliteConnection> {
        let mut db = SqliteConnection::connect("sqlite::memory:").await?;
        create_schema(&mut db).await?;
        let id_tx = chain_log::record_transaction(&mut db, 1, &TransactionRow::chain(&[9], 100, 0))
            .await?
            .unwrap();
        let mut nf = vec![];
        outpoint.write(&mut nf)?;
        chain_log::record_note(
            &mut db,
            1,
            &NoteRow {
                id_tx,
                height: 100,
                pool: 0,
                nullifier: &nf,
                value: 5000,
                taddress: Some(1),
                ..Default::default()
            },
        )
        .await?;
        Ok(db)
    }

    async fn locked(db: &mut SqliteConnection) -> Result<i64> {
        Ok(
            sqlx::query_scalar("SELECT COUNT(*) FROM notes WHERE account = 1 AND locked")
                .fetch_one(db)
                .await?,
        )
    }

    #[tokio::test]
    async fn an_accepted_broadcast_locks_the_spent_note_and_records_pending() -> Result<()> {
        let outpoint = OutPoint::new([7u8; 32], 3);
        let mut db = wallet_with_note(&outpoint).await?;
        let mut client: Client = Box::new(Posts(TXID.to_string(), vec![]));
        let pending = Pending {
            price: Some(1.5),
            category: Some(4),
        };
        let txid =
            broadcast_under_session(&mut db, &mut client, 1, &spend_of(outpoint), Some(pending))
                .await?;
        assert_eq!(txid, TXID);
        assert_eq!(locked(&mut db).await?, 1);
        let rows: Vec<(u32, Vec<u8>, Option<f64>, Option<u32>)> = sqlx::query_as(
            "SELECT height, txid, price, category FROM pending_txs WHERE account = 1",
        )
        .fetch_all(&mut db)
        .await?;
        let mut expected_txid = hex::decode(TXID).unwrap();
        expected_txid.reverse();
        assert_eq!(rows, vec![(500, expected_txid, Some(1.5), Some(4))]);
        Ok(())
    }

    #[tokio::test]
    async fn without_pending_metadata_nothing_is_recorded() -> Result<()> {
        let outpoint = OutPoint::new([7u8; 32], 3);
        let mut db = wallet_with_note(&outpoint).await?;
        let mut client: Client = Box::new(Posts(TXID.to_string(), vec![]));
        broadcast_under_session(&mut db, &mut client, 1, &spend_of(outpoint), None).await?;
        assert_eq!(locked(&mut db).await?, 1);
        let pending: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pending_txs")
            .fetch_one(&mut db)
            .await?;
        assert_eq!(pending, 0);
        Ok(())
    }

    #[tokio::test]
    async fn a_rejection_is_an_error_and_locks_nothing() -> Result<()> {
        let outpoint = OutPoint::new([7u8; 32], 3);
        let mut db = wallet_with_note(&outpoint).await?;
        let mut client: Client = Box::new(Posts("tx has expired".to_string(), vec![]));
        let err = broadcast_under_session(&mut db, &mut client, 1, &spend_of(outpoint), None)
            .await
            .unwrap_err();
        assert_eq!(err.to_string(), "tx has expired");
        assert_eq!(locked(&mut db).await?, 0);
        Ok(())
    }

    #[tokio::test]
    async fn unparseable_bytes_still_send_and_lock_nothing() -> Result<()> {
        let outpoint = OutPoint::new([7u8; 32], 3);
        let mut db = wallet_with_note(&outpoint).await?;
        let mut client: Client = Box::new(Posts(TXID.to_string(), vec![]));
        let txid =
            broadcast_under_session(&mut db, &mut client, 1, b"not a transaction", None).await?;
        assert_eq!(txid, TXID);
        assert_eq!(locked(&mut db).await?, 0);
        Ok(())
    }
}
