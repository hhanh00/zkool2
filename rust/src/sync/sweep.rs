//! Transparent address discovery: derive addresses past the ones the
//! wallet knows and keep the used ones, within a gap limit.

use anyhow::Result;
use sqlx::SqliteConnection;
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;

use crate::account::get_birth_height;
use crate::api::coin::Network;
use crate::db::store_account_transparent_addr;
use crate::transparent_keys::{ensure_transparent_address, TransparentKeySource};
use crate::Client;

/// Scan the account's transparent address chains, reporting each address
/// through `progress_fn` as it is tried, and return how many were added.
/// Runs to the end, or to the cancel, in the caller's task: its error is
/// the caller's to report, where a detached task dropped it (#1502).
#[allow(clippy::too_many_arguments)]
pub async fn transparent_sweep(
    network: &Network,
    connection: &mut SqliteConnection,
    client: &mut Client,
    account: u32,
    end_height: u32,
    gap_limit: u32,
    progress_fn: impl Fn(String),
    cancellation_token: CancellationToken,
) -> Result<usize> {
    let source = TransparentKeySource::load(&mut *connection, account).await?;
    let (use_internal,): (bool,) =
        sqlx::query_as("SELECT use_internal FROM accounts WHERE id_account = ?")
            .bind(account)
            .fetch_one(&mut *connection)
            .await?;
    let start_height = get_birth_height(&mut *connection, account).await?;
    scan_transparent_addresses(
        network,
        connection,
        client,
        account,
        &source,
        start_height,
        end_height,
        gap_limit,
        use_internal,
        progress_fn,
        cancellation_token,
    )
    .await
}

/// Walk the external and change chains from index 0, asking the server for
/// each address's history. A used address is stored together with its
/// counterpart on the other chain (external to change only when the account
/// uses internal change); the scan of a chain ends after `gap_limit`
/// consecutive unused addresses, a used one resetting the count. The
/// account's receiving index then moves up to the highest used external
/// index, never down. Returns how many addresses were added.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn scan_transparent_addresses(
    network: &Network,
    connection: &mut SqliteConnection,
    client: &mut Client,
    account: u32,
    source: &TransparentKeySource,
    start_height: u32,
    end_height: u32,
    gap_limit: u32,
    use_internal: bool,
    progress_fn: impl Fn(String),
    cancellation_token: CancellationToken,
) -> Result<usize> {
    let mut n_added = 0;
    // A cancel leaves both loops rather than the function, so the receiving
    // index below still moves past the addresses stored so far (#1491).
    'scan: for scope in 0..2u32 {
        let mut dindex = 0;
        let mut gap = 0;
        loop {
            let addr = source.derive(network, scope, dindex).await?;
            progress_fn(addr.address.clone());

            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    break 'scan;
                }

                txids = client
                    .taddress_txs(network, &addr.address, start_height, end_height)
                    => {
                    let mut txids = txids?;
                    // The first read is where a server that opens the stream
                    // and never answers parks the scan; it is raced against
                    // the token too, and dropping the stream ends the reader
                    // (#1481). A stream that fails before its first item is
                    // not an unused address (V12 F-77).
                    let first = tokio::select! {
                        _ = cancellation_token.cancelled() => break 'scan,
                        m = txids.next() => m,
                    };
                    let used = match first {
                        Some(Ok(_)) => true,
                        Some(Err(e)) => {
                            return Err(e.context(format!(
                                "the history of {} could not be read",
                                addr.address
                            )));
                        }
                        None => false,
                    };
                    if used {
                        // A used address resets the gap count: only
                        // consecutive unused addresses end the scan.
                        gap = 0;
                        let added = store_account_transparent_addr(
                            connection,
                            account,
                            scope,
                            dindex,
                            addr.sk,
                            &addr.pk,
                            &addr.address,
                            false,
                        )
                        .await?;
                        if added {
                            n_added += 1;
                        }

                        let want_counterpart = if scope == 0 { use_internal } else { true };
                        if want_counterpart {
                            ensure_transparent_address(
                                connection,
                                network,
                                source,
                                account,
                                1 - scope,
                                dindex,
                            )
                            .await?;
                        }
                    } else {
                        gap += 1;
                    }
                    dindex += 1;
                    if gap > gap_limit {
                        break;
                    }
                }
            }
        }
    }
    // Advance the account dindex to the greatest stored external (scope 0)
    // used index, never backwards. The scan only stores used addresses and
    // address generation keeps accounts.dindex at the newest row, so the
    // table's max external dindex is the receiving frontier.
    sqlx::query(
        "UPDATE accounts
        SET dindex = max(
            dindex,
            COALESCE(
                (SELECT max(dindex) FROM transparent_address_accounts
                 WHERE account = accounts.id_account AND scope = 0),
                0))
        WHERE id_account = ?",
    )
    .bind(account)
    .execute(&mut *connection)
    .await?;
    Ok(n_added)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::create_schema;
    use crate::lwd::CompactBlock;
    use crate::net::LwdServer;
    use crate::transparent_keys::tests::software_keys;
    use sqlx::Connection;
    use std::collections::HashSet;
    use tokio_stream::wrappers::ReceiverStream;
    use tonic::async_trait;
    use zcash_primitives::transaction::{Transaction, TransactionData, TxVersion};
    use zcash_protocol::consensus::{BlockHeight, BranchId};
    use zcash_protocol::value::Zatoshis;

    /// A lightwalletd that knows which addresses have history and nothing else.
    struct UsedAddresses(HashSet<String>);

    fn empty_tx() -> Transaction {
        TransactionData::<zcash_primitives::transaction::Authorized>::from_parts(
            TxVersion::V5,
            BranchId::Nu5,
            0,
            BlockHeight::from_u32(0),
            Zatoshis::ZERO,
            None,
            None,
            None,
            None,
        )
        .freeze()
        .unwrap()
    }

    #[async_trait]
    impl LwdServer for UsedAddresses {
        type CompactBlockStream = ReceiverStream<Result<CompactBlock>>;
        type TransactionStream = ReceiverStream<Result<(u32, Transaction, usize)>>;

        async fn latest_height(&mut self) -> Result<u32> {
            unimplemented!()
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
            taddress: &str,
            _: u32,
            _: u32,
        ) -> Result<Self::TransactionStream> {
            let (tx, rx) = tokio::sync::mpsc::channel(1);
            if self.0.contains(taddress) {
                tx.send(Ok((100, empty_tx(), 0))).await.unwrap();
            }
            Ok(ReceiverStream::new(rx))
        }
        async fn mempool_stream(&mut self, _: &Network) -> Result<Self::TransactionStream> {
            unimplemented!()
        }
        async fn tree_state(&mut self, _: u32) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
            unimplemented!()
        }
    }

    async fn test_db(use_internal: bool) -> Result<SqliteConnection> {
        let mut db = SqliteConnection::connect("sqlite::memory:").await?;
        create_schema(&mut db).await?;
        sqlx::query("INSERT INTO accounts(id_account, name, aindex, dindex, def_dindex, birth, position, use_internal, hidden, saved, hw)
            VALUES (1, 'a', 0, 2, 2, 1, 1, ?, FALSE, FALSE, 0)")
            .bind(use_internal)
            .execute(&mut db)
            .await?;
        crate::db::init_account_transparent(&mut db, 1, 1).await?;
        Ok(db)
    }

    async fn stored(db: &mut SqliteConnection) -> Result<Vec<(u32, u32)>> {
        let rows = sqlx::query_as(
            "SELECT scope, dindex FROM transparent_address_accounts WHERE account = 1 ORDER BY scope, dindex",
        )
        .fetch_all(db)
        .await?;
        Ok(rows)
    }

    async fn run(
        db: &mut SqliteConnection,
        used: &[(u32, u32)],
        gap_limit: u32,
        use_internal: bool,
    ) -> Result<(usize, Vec<String>)> {
        let (xvk, xsk) = software_keys();
        let source = TransparentKeySource::software(xvk, Some(xsk));
        let mut addresses = HashSet::new();
        for &(scope, dindex) in used {
            addresses.insert(source.derive(&Network::Main, scope, dindex).await?.address);
        }
        let mut client: Client = Box::new(UsedAddresses(addresses));
        let tried = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
        let seen = tried.clone();
        let added = scan_transparent_addresses(
            &Network::Main,
            db,
            &mut client,
            1,
            &source,
            1,
            1000,
            gap_limit,
            use_internal,
            move |a| seen.lock().unwrap().push(a),
            CancellationToken::new(),
        )
        .await?;
        let tried = tried.lock().unwrap().clone();
        Ok((added, tried))
    }

    /// A server whose history stream fails before its first item.
    struct BrokenHistory;

    #[async_trait]
    impl LwdServer for BrokenHistory {
        type CompactBlockStream = ReceiverStream<Result<CompactBlock>>;
        type TransactionStream = ReceiverStream<Result<(u32, Transaction, usize)>>;

        async fn latest_height(&mut self) -> Result<u32> {
            unimplemented!()
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
            let (tx, rx) = tokio::sync::mpsc::channel(1);
            tx.send(Err(anyhow::anyhow!("connection reset"))).await.unwrap();
            Ok(ReceiverStream::new(rx))
        }
        async fn mempool_stream(&mut self, _: &Network) -> Result<Self::TransactionStream> {
            unimplemented!()
        }
        async fn tree_state(&mut self, _: u32) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
            unimplemented!()
        }
    }

    // A history stream that dies is not an empty history (#1474, V12 F-77):
    // the sweep must not count the address as unused, and the transparent
    // scan must not checkpoint past transactions it never saw.

    #[tokio::test]
    async fn a_failed_history_stream_fails_the_sweep_instead_of_counting_as_unused() -> Result<()> {
        let mut db = test_db(false).await?;
        let (xvk, xsk) = software_keys();
        let source = TransparentKeySource::software(xvk, Some(xsk));
        let mut client: Client = Box::new(BrokenHistory);
        let r = scan_transparent_addresses(
            &Network::Main,
            &mut db,
            &mut client,
            1,
            &source,
            1,
            1000,
            2,
            false,
            |_| {},
            CancellationToken::new(),
        )
        .await;
        let err = format!("{:#}", r.unwrap_err());
        assert!(err.contains("could not be read"), "{err}");
        assert!(err.contains("connection reset"), "{err}");
        assert!(stored(&mut db).await?.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn a_failed_history_stream_fails_the_transparent_scan_and_keeps_the_checkpoint() -> Result<()> {
        let mut db = test_db(false).await?;
        let (xvk, xsk) = software_keys();
        let source = TransparentKeySource::software(xvk, Some(xsk));
        let addr = source.derive(&Network::Main, 0, 0).await?;
        crate::db::store_account_transparent_addr(&mut db, 1, 0, 0, addr.sk, &addr.pk, &addr.address, false)
            .await?;
        let mut client: Client = Box::new(BrokenHistory);
        let r = super::super::transparent::transparent_sync(
            &Network::Main,
            &mut db,
            &mut client,
            &[1],
            1,
            500,
            CancellationToken::new(),
        )
        .await;
        let err = format!("{:#}", r.unwrap_err());
        assert!(err.contains("failed") && err.contains("connection reset"), "{err}");
        let (h,): (u32,) =
            sqlx::query_as("SELECT height FROM sync_heights WHERE account = 1 AND pool = 0")
                .fetch_one(&mut db)
                .await?;
        assert_eq!(h, 1, "the checkpoint stays where the stream left it");
        Ok(())
    }

    /// A lightwalletd that cancels the sweep when asked about one address,
    /// as a Stop pressed at that moment would.
    struct CancelAt {
        used: HashSet<String>,
        at: String,
        token: CancellationToken,
    }

    #[async_trait]
    impl LwdServer for CancelAt {
        type CompactBlockStream = ReceiverStream<Result<CompactBlock>>;
        type TransactionStream = ReceiverStream<Result<(u32, Transaction, usize)>>;

        async fn latest_height(&mut self) -> Result<u32> {
            unimplemented!()
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
            taddress: &str,
            _: u32,
            _: u32,
        ) -> Result<Self::TransactionStream> {
            if taddress == self.at {
                self.token.cancel();
            }
            let (tx, rx) = tokio::sync::mpsc::channel(1);
            if self.used.contains(taddress) {
                tx.send(Ok((100, empty_tx(), 0))).await.unwrap();
            }
            Ok(ReceiverStream::new(rx))
        }
        async fn mempool_stream(&mut self, _: &Network) -> Result<Self::TransactionStream> {
            unimplemented!()
        }
        async fn tree_state(&mut self, _: u32) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
            unimplemented!()
        }
    }

    /// #1491: a cancelled sweep still moves the receiving index past the
    /// used addresses it stored before the stop.
    #[tokio::test]
    async fn a_cancelled_sweep_still_moves_the_receiving_index() -> Result<()> {
        let mut db = test_db(false).await?;
        let (xvk, xsk) = software_keys();
        let source = TransparentKeySource::software(xvk, Some(xsk));
        let token = CancellationToken::new();
        let used: HashSet<String> = [source.derive(&Network::Main, 0, 5).await?.address]
            .into_iter()
            .collect();
        // Stop arrives while index 6 is being asked about, after 5 is stored.
        let at = source.derive(&Network::Main, 0, 6).await?.address;
        let mut client: Client = Box::new(CancelAt { used, at, token: token.clone() });
        let added = scan_transparent_addresses(
            &Network::Main,
            &mut db,
            &mut client,
            1,
            &source,
            1,
            1000,
            10,
            false,
            |_| {},
            token,
        )
        .await?;
        assert_eq!(added, 1);
        assert_eq!(stored(&mut db).await?, vec![(0, 5)]);
        let (dindex,): (u32,) = sqlx::query_as("SELECT dindex FROM accounts WHERE id_account = 1")
            .fetch_one(&mut db)
            .await?;
        assert_eq!(dindex, 5, "the receiving index moved up although the sweep was stopped");
        Ok(())
    }

    /// A server that opens every history stream and never answers on it.
    struct SilentHistory;

    #[async_trait]
    impl LwdServer for SilentHistory {
        type CompactBlockStream = ReceiverStream<Result<CompactBlock>>;
        type TransactionStream = ReceiverStream<Result<(u32, Transaction, usize)>>;

        async fn latest_height(&mut self) -> Result<u32> {
            unimplemented!()
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
            let (tx, rx) = tokio::sync::mpsc::channel(1);
            // The sender outlives the stream, never sending: a read waits
            // for good.
            tokio::spawn(async move {
                let _keep = tx;
                std::future::pending::<()>().await;
            });
            Ok(ReceiverStream::new(rx))
        }
        async fn mempool_stream(&mut self, _: &Network) -> Result<Self::TransactionStream> {
            unimplemented!()
        }
        async fn tree_state(&mut self, _: u32) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
            unimplemented!()
        }
    }

    /// #1481: a sweep parked on a history stream that never answers ends
    /// when it is cancelled.
    #[tokio::test]
    async fn a_cancel_ends_a_sweep_whose_stream_never_yields() -> Result<()> {
        let mut db = test_db(false).await?;
        let (xvk, xsk) = software_keys();
        let source = TransparentKeySource::software(xvk, Some(xsk));
        let token = CancellationToken::new();
        let cancel = token.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            cancel.cancel();
        });
        let mut client: Client = Box::new(SilentHistory);
        let scan = scan_transparent_addresses(
            &Network::Main,
            &mut db,
            &mut client,
            1,
            &source,
            1,
            1000,
            2,
            false,
            |_| {},
            token,
        );
        let added = tokio::time::timeout(std::time::Duration::from_secs(5), scan)
            .await
            .expect("the cancelled sweep returns")?;
        assert_eq!(added, 0);
        Ok(())
    }

    /// A lightwalletd with no history that remembers which addresses it was
    /// asked about.
    struct AskedAddresses(std::sync::Arc<std::sync::Mutex<Vec<String>>>);

    #[async_trait]
    impl LwdServer for AskedAddresses {
        type CompactBlockStream = ReceiverStream<Result<CompactBlock>>;
        type TransactionStream = ReceiverStream<Result<(u32, Transaction, usize)>>;

        async fn latest_height(&mut self) -> Result<u32> {
            unimplemented!()
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
            taddress: &str,
            _: u32,
            _: u32,
        ) -> Result<Self::TransactionStream> {
            self.0.lock().unwrap().push(taddress.to_string());
            let (_tx, rx) = tokio::sync::mpsc::channel(1);
            Ok(ReceiverStream::new(rx))
        }
        async fn mempool_stream(&mut self, _: &Network) -> Result<Self::TransactionStream> {
            unimplemented!()
        }
        async fn tree_state(&mut self, _: u32) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
            unimplemented!()
        }
    }

    /// #1501: every stored address is queried on every range, so the
    /// checkpoint the scan writes covers all of them.
    #[tokio::test]
    async fn the_transparent_scan_asks_about_every_stored_address() -> Result<()> {
        let mut db = test_db(true).await?;
        let (xvk, xsk) = software_keys();
        let source = TransparentKeySource::software(xvk, Some(xsk));
        let mut expected = vec![];
        for (scope, dindex) in [(0u32, 0u32), (0, 1), (0, 2), (1, 0)] {
            let addr = source.derive(&Network::Main, scope, dindex).await?;
            crate::db::store_account_transparent_addr(&mut db, 1, scope, dindex, addr.sk, &addr.pk, &addr.address, false)
                .await?;
            expected.push(addr.address);
        }
        let asked = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
        let mut client: Client = Box::new(AskedAddresses(asked.clone()));
        super::super::transparent::transparent_sync(
            &Network::Main,
            &mut db,
            &mut client,
            &[1],
            1,
            500,
            CancellationToken::new(),
        )
        .await?;
        let mut asked = asked.lock().unwrap().clone();
        asked.sort();
        expected.sort();
        assert_eq!(asked, expected);
        let (h,): (u32,) =
            sqlx::query_as("SELECT height FROM sync_heights WHERE account = 1 AND pool = 0")
                .fetch_one(&mut db)
                .await?;
        assert_eq!(h, 500);
        Ok(())
    }

    /// #1571: a cancel that landed before the scanner started is seen by
    /// it; nothing is asked of the server.
    #[tokio::test]
    async fn a_cancelled_token_stops_the_transparent_scan_before_any_request() -> Result<()> {
        let mut db = test_db(false).await?;
        let (xvk, xsk) = software_keys();
        let source = TransparentKeySource::software(xvk, Some(xsk));
        let addr = source.derive(&Network::Main, 0, 0).await?;
        crate::db::store_account_transparent_addr(&mut db, 1, 0, 0, addr.sk, &addr.pk, &addr.address, false)
            .await?;
        let cancel = CancellationToken::new();
        cancel.cancel();
        let mut client: Client = Box::new(BrokenHistory);
        let r = super::super::transparent::transparent_sync(
            &Network::Main,
            &mut db,
            &mut client,
            &[1],
            1,
            500,
            cancel,
        )
        .await;
        let err = format!("{:#}", r.unwrap_err());
        assert!(err.contains("Sync canceled"), "{err}");
        assert!(!err.contains("connection reset"), "the server was not asked: {err}");
        Ok(())
    }

    #[tokio::test]
    async fn stops_after_the_gap_limit_on_each_chain() -> Result<()> {
        let mut db = test_db(false).await?;
        let (added, tried) = run(&mut db, &[], 2, false).await?;
        assert_eq!(added, 0);
        // indexes 0, 1, 2 on each of the two chains: gap 3 > limit 2
        assert_eq!(tried.len(), 6);
        assert!(stored(&mut db).await?.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn a_used_address_after_unused_ones_resets_the_gap() -> Result<()> {
        let mut db = test_db(false).await?;
        // external 0 and 3 used: 1 and 2 are a gap of 2, within the limit
        let (added, tried) = run(&mut db, &[(0, 0), (0, 3)], 2, false).await?;
        assert_eq!(added, 2);
        // external: 0..=6 (3 unused after index 3), change: 0..=2
        assert_eq!(tried.len(), 7 + 3);
        assert_eq!(stored(&mut db).await?, vec![(0, 0), (0, 3)]);
        Ok(())
    }

    #[tokio::test]
    async fn pairs_change_with_external_only_for_internal_change_accounts() -> Result<()> {
        let mut db = test_db(true).await?;
        let (added, _) = run(&mut db, &[(0, 1)], 1, true).await?;
        assert_eq!(added, 1);
        assert_eq!(stored(&mut db).await?, vec![(0, 1), (1, 1)]);

        let mut db = test_db(false).await?;
        run(&mut db, &[(0, 1)], 1, false).await?;
        assert_eq!(stored(&mut db).await?, vec![(0, 1)]);
        Ok(())
    }

    #[tokio::test]
    async fn a_used_change_address_always_gets_its_external_twin() -> Result<()> {
        let mut db = test_db(false).await?;
        run(&mut db, &[(1, 4)], 5, false).await?;
        assert_eq!(stored(&mut db).await?, vec![(0, 4), (1, 4)]);
        Ok(())
    }

    #[tokio::test]
    async fn the_receiving_index_moves_up_to_the_highest_used_external() -> Result<()> {
        let mut db = test_db(false).await?;
        run(&mut db, &[(0, 5)], 5, false).await?;
        let dindex: u32 = sqlx::query_scalar("SELECT dindex FROM accounts WHERE id_account = 1")
            .fetch_one(&mut db)
            .await?;
        assert_eq!(dindex, 5);
        // a scan finding only lower indexes never moves it back
        run(&mut db, &[(0, 1)], 5, false).await?;
        let dindex: u32 = sqlx::query_scalar("SELECT dindex FROM accounts WHERE id_account = 1")
            .fetch_one(&mut db)
            .await?;
        assert_eq!(dindex, 5);
        Ok(())
    }
}
