//! Transparent address discovery: derive addresses past the ones the
//! wallet knows and keep the used ones, within a gap limit.

use anyhow::Result;
use tokio_stream::StreamExt;

use crate::api::account::get_ledger;
use crate::api::coin::Network;

use crate::Client;

use crate::{
    account::{derive_transparent_address, derive_transparent_sk, get_birth_height},
    db::{
        get_account_aindex, get_account_dindex, get_account_hw, select_account_transparent,
        store_account_transparent_addr,
    },
};
use sqlx::pool::PoolConnection;
use sqlx::Sqlite;

use tokio_util::sync::CancellationToken;
use zcash_keys::encoding::AddressCodec;

#[allow(clippy::too_many_arguments)]
pub async fn transparent_sweep(
    network: &Network,
    mut connection: PoolConnection<Sqlite>,
    mut client: Client,
    account: u32,
    end_height: u32,
    gap_limit: u32,
    progress_fn: impl Fn(String) + 'static + Send,
    cancellation_token: CancellationToken,
) -> Result<()> {
    let network = *network;
    let hw = get_account_hw(&mut connection, account).await?;
    let aindex = get_account_aindex(&mut connection, account).await?;
    let dindex = get_account_dindex(&mut connection, account).await?;
    let (use_internal,): (bool,) =
        sqlx::query_as("SELECT use_internal FROM accounts WHERE id_account = ?")
            .bind(account)
            .fetch_one(&mut *connection)
            .await?;
    tokio::spawn(async move {
        let ledger = get_ledger(&mut connection, account).await?;
        let mut n_added = 0;
        let tk = select_account_transparent(&mut connection, account, dindex).await?;
        let xvk = tk.xvk;
        let start_height = get_birth_height(&mut connection, account).await?;
        for scope in 0..2 {
            let mut dindex = 0;
            let mut gap = 0;
            loop {
                let (pk, taddr) = match xvk.as_ref() {
                    Some(xvk) => derive_transparent_address(xvk, scope, dindex, false)?,
                    None if hw != 0 => {
                        ledger
                            .get_transparent_pubkey(&network, aindex, scope, dindex)
                            .await?
                    }
                    _ => anyhow::bail!("Sweep needs an xpub key"),
                };
                let taddr = taddr.encode(&network);
                progress_fn(taddr.clone());

                tokio::select! {
                    _ = cancellation_token.cancelled() => {
                        return Ok::<_, anyhow::Error>(n_added)
                    }

                    txids = client
                        .taddress_txs(&network, &taddr, start_height, end_height)
                        => {
                        let mut txids = txids?;
                        if txids.next().await.is_some() {
                            // A used address resets the gap count: only
                            // consecutive unused addresses end the scan.
                            gap = 0;
                            let sk = if let Some(tsk) = tk.xsk.as_ref() {
                                Some(derive_transparent_sk(tsk, scope, dindex)?)
                            } else {
                                None
                            };
                            let added = store_account_transparent_addr(
                                &mut connection, account, scope, dindex, sk, &pk, &taddr, false,
                            )
                            .await?;
                            if added {
                                n_added += 1;
                            }

                            let want_counterpart = if scope == 0 {
                                use_internal
                            } else {
                                true
                            };
                            if want_counterpart {
                                let counterpart_scope = 1 - scope;
                                let existing: (i64,) = sqlx::query_as(
                                    "SELECT COUNT(*) FROM transparent_address_accounts
                                    WHERE account = ?1 AND scope = ?2 AND dindex = ?3",
                                )
                                .bind(account)
                                .bind(counterpart_scope)
                                .bind(dindex)
                                .fetch_one(&mut *connection)
                                .await?;
                                if existing.0 == 0 {
                                    let (csk, cpk, ctaddr) = match xvk.as_ref() {
                                        Some(xvk) => {
                                            let sk = if let Some(tsk) = tk.xsk.as_ref() {
                                                Some(derive_transparent_sk(
                                                    tsk,
                                                    counterpart_scope,
                                                    dindex,
                                                )?)
                                            } else {
                                                None
                                            };
                                            let (pk, address) = derive_transparent_address(
                                                xvk,
                                                counterpart_scope,
                                                dindex,
                                                false,
                                            )?;
                                            (sk, pk, address)
                                        }
                                        None if hw != 0 => {
                                            let (pk, address) = ledger
                                                .get_transparent_pubkey(
                                                    &network,
                                                    aindex,
                                                    counterpart_scope,
                                                    dindex,
                                                )
                                                .await?;
                                            (None, pk, address)
                                        }
                                        _ => anyhow::bail!("Sweep needs an xpub key"),
                                    };
                                    store_account_transparent_addr(
                                        &mut connection,
                                        account,
                                        counterpart_scope,
                                        dindex,
                                        csk,
                                        &cpk,
                                        &ctaddr.encode(&network),
                                        false,
                                    )
                                    .await?;
                                }
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
    });
    Ok(())
}
