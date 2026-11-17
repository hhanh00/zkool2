//! Chain continuity at range boundaries (#1658).
//!
//! The shielded scan compares every block it streams with the one before,
//! but a range whose accounts have no shielded keys streams no blocks, and
//! the transparent scan only sees transaction heights. So every range
//! starts by comparing the header each account holds at `start - 1` with
//! the server's block there, and ends by storing the server's block at
//! `end`, which the next range or the next run compares in turn.

use std::collections::HashMap;

use anyhow::Result;
use sqlx::{Connection, SqliteConnection};
use tracing::info;

use super::chain_log;
use super::{rewind_within, BlockHeader};
use crate::api::coin::Network;
use crate::net::LwdServer;

/// The server's blocks fetched during one run, by height. A range's `end`
/// is the next range's `start - 1`, so each height is fetched once.
#[derive(Default)]
pub(crate) struct ServerHeaders(HashMap<u32, (Vec<u8>, u32)>);

impl ServerHeaders {
    pub(crate) async fn get<C: LwdServer + ?Sized>(
        &mut self,
        client: &mut C,
        network: &Network,
        height: u32,
    ) -> Result<BlockHeader> {
        let (hash, time) = match self.0.get(&height) {
            Some(fetched) => fetched.clone(),
            None => {
                let block = client.block(network, height).await?;
                self.0.insert(height, (block.hash.clone(), block.time));
                (block.hash, block.time)
            }
        };
        Ok(BlockHeader { height, hash, time })
    }
}

/// Compare what the range's accounts hold at `start - 1` with the server's
/// block there. On a mismatch, rewind every account of the range below
/// `start - 1` and return the height the fork was found at, `start`, for
/// the caller to plan again. An account without a header there (its first
/// range after #1658, or a new account at its birth) is not checked.
///
/// A mismatch at a final block fails and trims nothing, as in the shielded
/// scan.
pub(crate) async fn check_range_start<C: LwdServer + ?Sized>(
    network: &Network,
    connection: &mut SqliteConnection,
    client: &mut C,
    headers: &mut ServerHeaders,
    accounts: &[u32],
    start: u32,
    tip: u32,
) -> Result<Option<u32>> {
    let Some(below) = start.checked_sub(1) else {
        return Ok(None);
    };
    let mut held = vec![];
    for account in accounts {
        if let Some(hash) = chain_log::header_hash(connection, *account, below).await? {
            held.push(hash);
        }
    }
    if held.is_empty() {
        return Ok(None);
    }
    let server = headers.get(client, network, below).await?;
    if held.iter().all(|hash| *hash == server.hash) {
        return Ok(None);
    }
    if chain_log::is_final(below, tip) {
        anyhow::bail!(
            "the server's block {below} differs from the final block {below} \
             the accounts hold (tip {tip}): not rewinding"
        );
    }
    info!("the server's block {below} differs from the one the accounts hold: rewinding");
    let mut db_tx = connection.begin().await?;
    for account in accounts {
        rewind_within(network, &mut db_tx, *account, below).await?;
    }
    db_tx.commit().await?;
    Ok(Some(start))
}

/// Store the server's block at the end of the range for the accounts whose
/// sync height reached it, for the next range to check.
pub(crate) async fn record_range_end(
    connection: &mut SqliteConnection,
    accounts: &[u32],
    header: &BlockHeader,
) -> Result<()> {
    for account in accounts {
        chain_log::record_relevant_header(connection, *account, header).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::create_schema;
    use crate::lwd::CompactBlock;
    use chain_log::{set_checkpoint, store_header, FINALITY_DEPTH};
    use tokio_stream::wrappers::ReceiverStream;
    use tonic::async_trait;
    use zcash_primitives::transaction::Transaction;

    /// A server whose block at every height has hash `[height, fork]`.
    /// Records the heights it was asked for.
    struct Chain {
        fork: u8,
        asked: Vec<u32>,
    }

    fn hash(height: u32, fork: u8) -> Vec<u8> {
        let mut v = vec![fork; 32];
        v[..4].copy_from_slice(&height.to_le_bytes());
        v
    }

    #[async_trait]
    impl LwdServer for Chain {
        type CompactBlockStream = ReceiverStream<Result<CompactBlock>>;
        type TransactionStream = ReceiverStream<Result<(u32, Transaction, usize)>>;

        async fn latest_height(&mut self) -> Result<u32> {
            unimplemented!()
        }
        async fn block(&mut self, _: &Network, height: u32) -> Result<CompactBlock> {
            self.asked.push(height);
            Ok(CompactBlock {
                height: height as u64,
                hash: hash(height, self.fork),
                time: height,
                ..Default::default()
            })
        }
        async fn block_range(&mut self, _: &Network, _: u32, _: u32) -> Result<Self::CompactBlockStream> {
            unimplemented!()
        }
        async fn transaction(&mut self, _: &Network, _: &[u8]) -> Result<(u32, Transaction)> {
            unimplemented!()
        }
        async fn post_transaction(&mut self, _: u32, _: &[u8]) -> Result<String> {
            unimplemented!()
        }
        async fn taddress_txs(&mut self, _: &Network, _: &str, _: u32, _: u32) -> Result<Self::TransactionStream> {
            unimplemented!()
        }
        async fn mempool_stream(&mut self, _: &Network) -> Result<Self::TransactionStream> {
            unimplemented!()
        }
        async fn tree_state(&mut self, _: u32) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
            unimplemented!()
        }
    }

    fn chain(fork: u8) -> Chain {
        Chain { fork, asked: vec![] }
    }

    /// Transparent-only accounts: only pool 0 has a checkpoint.
    async fn wallet(accounts: &[u32], height: u32) -> Result<SqliteConnection> {
        let mut db = SqliteConnection::connect("sqlite::memory:").await?;
        create_schema(&mut db).await?;
        for account in accounts {
            set_checkpoint(&mut db, *account, 0, height).await?;
        }
        Ok(db)
    }

    async fn hold(db: &mut SqliteConnection, account: u32, heights: &[u32], fork: u8) -> Result<()> {
        for &height in heights {
            let header = BlockHeader { height, hash: hash(height, fork), time: height };
            store_header(db, account, &header).await?;
        }
        Ok(())
    }

    async fn synced_at(db: &mut SqliteConnection, account: u32) -> Result<u32> {
        let h = sqlx::query_scalar("SELECT height FROM sync_heights WHERE account = ? AND pool = 0")
            .bind(account)
            .fetch_one(&mut *db)
            .await?;
        Ok(h)
    }

    async fn check(db: &mut SqliteConnection, server: &mut Chain, accounts: &[u32], start: u32, tip: u32) -> Result<Option<u32>> {
        let mut headers = ServerHeaders::default();
        check_range_start(&Network::Main, db, server, &mut headers, accounts, start, tip).await
    }

    #[tokio::test]
    async fn a_transparent_only_range_on_the_same_chain_passes() -> Result<()> {
        let mut db = wallet(&[1], 300).await?;
        hold(&mut db, 1, &[200, 300], 0).await?;
        let mut server = chain(0);
        assert_eq!(check(&mut db, &mut server, &[1], 301, 310).await?, None);
        assert_eq!(server.asked, vec![300]);
        assert_eq!(synced_at(&mut db, 1).await?, 300);
        Ok(())
    }

    #[tokio::test]
    async fn a_fork_below_the_range_rewinds_to_the_previous_header() -> Result<()> {
        let mut db = wallet(&[1, 2], 300).await?;
        hold(&mut db, 1, &[200, 300], 0).await?;
        hold(&mut db, 2, &[250, 300], 0).await?;
        let mut server = chain(1);
        assert_eq!(check(&mut db, &mut server, &[1, 2], 301, 310).await?, Some(301));
        assert_eq!(synced_at(&mut db, 1).await?, 200);
        assert_eq!(synced_at(&mut db, 2).await?, 250);
        let held: Vec<u32> = sqlx::query_scalar("SELECT height FROM headers WHERE height > 250")
            .fetch_all(&mut db)
            .await?;
        assert!(held.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn a_fork_at_a_final_block_fails_and_trims_nothing() -> Result<()> {
        let mut db = wallet(&[1], 300).await?;
        hold(&mut db, 1, &[200, 300], 0).await?;
        let mut server = chain(1);
        let err = check(&mut db, &mut server, &[1], 301, 300 + FINALITY_DEPTH)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("final block 300"), "{err}");
        assert_eq!(synced_at(&mut db, 1).await?, 300);
        Ok(())
    }

    #[tokio::test]
    async fn an_account_without_a_header_below_the_range_is_not_checked() -> Result<()> {
        let mut db = wallet(&[1], 300).await?;
        hold(&mut db, 1, &[200], 0).await?;
        let mut server = chain(1);
        assert_eq!(check(&mut db, &mut server, &[1], 301, 310).await?, None);
        assert!(server.asked.is_empty());
        assert_eq!(synced_at(&mut db, 1).await?, 300);
        Ok(())
    }

    #[tokio::test]
    async fn each_height_is_fetched_once_per_run() -> Result<()> {
        let mut server = chain(0);
        let mut headers = ServerHeaders::default();
        for height in [300, 310, 300] {
            headers.get(&mut server, &Network::Main, height).await?;
        }
        assert_eq!(server.asked, vec![300, 310]);
        Ok(())
    }

    #[tokio::test]
    async fn the_range_end_is_stored_where_the_sync_height_reached_it() -> Result<()> {
        let mut db = wallet(&[1, 2], 300).await?;
        set_checkpoint(&mut db, 1, 0, 310).await?;
        let header = BlockHeader { height: 310, hash: hash(310, 0), time: 310 };
        record_range_end(&mut db, &[1, 2], &header).await?;
        assert_eq!(chain_log::header_hash(&mut db, 1, 310).await?, Some(hash(310, 0)));
        assert_eq!(chain_log::header_hash(&mut db, 2, 310).await?, None);
        Ok(())
    }
}
