use std::{collections::HashMap, sync::LazyLock, time::Duration};

use crate::graphql::{check_admin_auth, check_auth, invalid_integer};
use crate::{
    account::generate_next_dindex,
    api::mempool::MempoolMsg,
    graphql::{
        data::{Addresses, Event, EventType, FrostRestoreResult, MigrationEvent, UnconfirmedNote},
        query::{prepare_tx, zats_to_zec},
        subs::{fan_out, SUBS},
        Context,
    },
};
use bigdecimal::BigDecimal;
use juniper::{graphql_object, FieldResult, GraphQLInputObject, GraphQLObject};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

pub struct Mutation {}

#[derive(GraphQLInputObject)]
pub struct NewAccount {
    pub name: String,
    /// Omit to import the public keys from a Ledger device when `hw` is set.
    pub key: Option<String>,
    pub passphrase: Option<String>,
    pub aindex: i32,
    pub birth: Option<i32>,
    pub pools: Option<i32>,
    pub use_internal: Option<bool>,
    /// Hardware wallet kind: 1 for the legacy Zondax app, 2 for the Official app.
    pub hw: Option<i32>,
}

#[derive(GraphQLInputObject)]
pub struct UpdateAccount {
    pub name: Option<String>,
    pub birth: Option<i32>,
    /// Use an internal address for change. Changing it resets the account's
    /// sync data. Every member of a FROST group must use the same setting: a
    /// member with it off does not see change sent to internal addresses.
    pub use_internal: Option<bool>,
}

#[derive(GraphQLInputObject)]
pub struct Recipient {
    pub address: String,
    pub amount: BigDecimal,
    pub memo: Option<String>,
    pub asset_desc: Option<String>,
}

#[derive(GraphQLInputObject)]
pub struct Payment {
    pub recipients: Vec<Recipient>,
    pub src_pools: Option<i32>,
    pub recipient_pays_fee: Option<bool>,
    pub confirmations: Option<i32>,
}

#[derive(GraphQLObject)]
pub struct Output {
    pub id: i32,
    pub pool: i32,
    pub vout: i32,
    pub value: BigDecimal,
    pub address: String,
    pub memo: Option<String>,
}

#[derive(GraphQLObject)]
pub struct UnsignedTx {
    pub recipients: Vec<Output>,
    pub fee: BigDecimal,
    /// The txid the transaction will have once signed, in the form
    /// broadcastTx returns. Signing does not change it; preparing the same
    /// payment again builds a new transaction with a new txid. Null when
    /// the PCZT's effects are not final, or for a transaction before v5.
    pub txid: Option<String>,
}

/// The height a sync mutation reports. A skipped run is an error: the
/// caller asked for a sync and none happened.
/// Sync through the wallet's driver, without progress: the mutation only
/// answers once the sync has ended.
async fn synchronize(
    coin: &crate::api::coin::Coin,
    accounts: Vec<u32>,
    current_height: u32,
    fast: bool,
) -> anyhow::Result<crate::api::sync::SyncOutcome> {
    let (progress, _) = tokio::sync::mpsc::channel(1);
    coin.driver()?
        .sync(
            coin,
            crate::sync::driver::SyncRequest {
                accounts,
                current_height,
                actions_per_sync: 100_000,
                checkpoint_age: 10_000,
                skip_details: fast,
            },
            progress,
        )
        .await
}

fn synced_height(outcome: crate::api::sync::SyncOutcome, current_height: u32) -> FieldResult<i32> {
    use crate::api::sync::SyncOutcome;
    match outcome {
        SyncOutcome::Synced { height } => Ok(height as i32),
        SyncOutcome::Cancelled => Ok(current_height as i32),
        SyncOutcome::Skipped => Err("a synchronization is already running".into()),
    }
}

#[graphql_object]
#[graphql(
    context = Context,
)]
impl Mutation {
    /// Permanently delete all closed swaps for the specified account.
    async fn near_intents_clear_closed_swaps(
        id_account: i32,
        context: &Context,
    ) -> FieldResult<bool> {
        if id_account <= 0 {
            return Err("Invalid account ID".into());
        }
        check_auth(context, id_account, true)?;
        crate::api::near_intents::near_intents_clear_closed_swaps(id_account as u32, &context.coin)
            .await?;
        Ok(true)
    }

    /// Permanently delete one swap owned by the specified account.
    async fn near_intents_delete_closed_swap(
        id_account: i32,
        id_swap: juniper::ID,
        context: &Context,
    ) -> FieldResult<bool> {
        if id_account <= 0 {
            return Err("Invalid account ID".into());
        }
        check_auth(context, id_account, true)?;
        let id_swap: i64 = id_swap.parse()?;
        if id_swap <= 0 {
            return Err("Invalid swap ID".into());
        }
        crate::api::near_intents::near_intents_delete_closed_swap(
            id_swap,
            id_account as u32,
            &context.coin,
        )
        .await?;
        Ok(true)
    }

    /// Refresh the provider status and persist it for a saved swap.
    async fn near_intents_refresh_swap_status(
        id_account: i32,
        id_swap: juniper::ID,
        context: &Context,
    ) -> FieldResult<crate::near_intents::SwapStatus> {
        if id_account <= 0 {
            return Err(juniper::FieldError::new(
                "Invalid account ID",
                juniper::Value::Null,
            ));
        }
        check_auth(context, id_account, true)?;
        let id_swap: i64 = id_swap.parse()?;
        if id_swap <= 0 {
            return Err(juniper::FieldError::new(
                "Invalid swap ID",
                juniper::Value::Null,
            ));
        }
        Ok(crate::api::near_intents::near_intents_refresh_swap_status(
            id_swap,
            id_account as u32,
            &context.coin,
        )
        .await?)
    }

    /// Preview with dry=true, then create a deposit quote with dry=false.
    async fn near_intents_quote(
        request: crate::near_intents::SwapRequest,
        context: &Context,
    ) -> FieldResult<crate::near_intents::SwapQuoteResponse> {
        check_admin_auth(context)?;
        Ok(crate::api::near_intents::near_intents_quote(request, &context.coin).await?)
    }

    /// Notify the provider after sending ZEC through the normal payment API.
    async fn near_intents_submit_deposit(
        deposit_address: String,
        tx_hash: String,
        context: &Context,
    ) -> FieldResult<bool> {
        check_admin_auth(context)?;
        crate::api::near_intents::near_intents_submit_deposit(
            deposit_address,
            tx_hash,
            &context.coin,
        )
        .await?;
        Ok(true)
    }

    /// Sets the authenticated static voting-config URL used by this wallet.
    async fn set_voting_config_url(url: String, context: &Context) -> FieldResult<bool> {
        check_admin_auth(context)?;
        let url = url.trim();
        if url.is_empty() {
            return Err(juniper::FieldError::new(
                "voting config URL cannot be empty",
                juniper::Value::Null,
            ));
        }
        if !(url.starts_with("http://") || url.starts_with("https://")) {
            return Err(juniper::FieldError::new(
                "voting config URL must use HTTP or HTTPS",
                juniper::Value::Null,
            ));
        }
        crate::api::db::put_prop("voting_config_url", url, &context.coin).await?;
        Ok(true)
    }

    /// Persists ballot choices and starts resumable submission in the background.
    async fn submit_vote(
        id_account: i32,
        round_id: String,
        selections: Vec<crate::graphql::voting::VotingSelectionInput>,
        context: &Context,
    ) -> FieldResult<crate::graphql::voting::VotingSubmissionStatus> {
        crate::graphql::voting::submit_vote(id_account, round_id, selections, context).await
    }

    async fn create_account(new_account: NewAccount, context: &Context) -> FieldResult<i32> {
        check_admin_auth(context)?;
        // GraphQL integers are signed; a negative one is refused here
        // rather than wrapped into a height or index near u32::MAX (#1532).
        let aindex = u32::try_from(new_account.aindex).map_err(|_| invalid_integer("aindex"))?;
        let birth = match new_account.birth {
            Some(birth) => u32::try_from(birth).map_err(|_| invalid_integer("birth"))?,
            None => crate::api::network::get_current_height(&context.coin).await?,
        };
        let na = crate::api::account::NewAccount {
            name: new_account.name,
            restore: false,
            key: new_account.key.unwrap_or_default(),
            passphrase: new_account.passphrase,
            fingerprint: None,
            icon: None,
            aindex,
            birth: Some(birth),
            pools: new_account.pools.map(|v| v as u8),
            use_internal: new_account.use_internal,
            folder: String::new(),
            internal: false,
            hw: new_account.hw.unwrap_or(0) as u8,
        };
        let id_account = crate::api::account::new_account(&na, &context.coin).await?;
        Ok(id_account as i32)
    }

    async fn edit_account(
        id_account: i32,
        update_account: UpdateAccount,
        context: &Context,
    ) -> FieldResult<bool> {
        check_auth(context, id_account, true)?;
        let internal_changes = match update_account.use_internal {
            Some(use_internal) => {
                let mut connection = context.coin.get_connection().await?;
                let (stored,): (bool,) =
                    sqlx::query_as("SELECT use_internal FROM accounts WHERE id_account = ?")
                        .bind(id_account)
                        .fetch_one(&mut *connection)
                        .await?;
                stored != use_internal
            }
            None => false,
        };
        let ua = crate::api::account::AccountUpdate {
            coin: 0,
            id: id_account as u32,
            name: update_account.name,
            icon: None,
            birth: update_account
                .birth
                .map(|v| u32::try_from(v).map_err(|_| invalid_integer("birth")))
                .transpose()?,
            folder: 0,
            hidden: None,
            enabled: None,
            use_internal: update_account.use_internal,
        };
        crate::api::account::update_account(&ua, &context.coin).await?;
        if internal_changes {
            crate::api::account::reset_sync(id_account as u32, &context.coin).await?;
        }
        Ok(true)
    }

    async fn delete_account(id_account: i32, context: &Context) -> FieldResult<bool> {
        check_auth(context, id_account, true)?;
        crate::api::account::delete_account(id_account as u32, &context.coin).await?;
        Ok(true)
    }

    async fn reset_account(id_account: i32, context: &Context) -> FieldResult<bool> {
        check_auth(context, id_account, true)?;
        crate::api::account::reset_sync(id_account as u32, &context.coin).await?;
        Ok(true)
    }

    /// Every stored transparent address is scanned on every range; the
    /// former `transparentLimit` window is gone (#1501).
    async fn synchronize(
        id_accounts: Vec<i32>,
        fast: Option<bool>,
        context: &Context,
    ) -> FieldResult<i32> {
        check_admin_auth(context)?;
        let fast = fast.unwrap_or_default();
        let id_accounts = id_accounts.into_iter().map(|v| v as u32).collect();
        let current_height = crate::api::network::get_current_height(&context.coin).await?;
        let outcome = synchronize(&context.coin, id_accounts, current_height, fast).await?;
        synced_height(outcome, current_height)
    }

    async fn synchronize_account(
        id_account: i32,
        fast: Option<bool>,
        context: &Context,
    ) -> FieldResult<i32> {
        check_auth(context, id_account, false)?;
        let fast = fast.unwrap_or_default();
        let current_height = crate::api::network::get_current_height(&context.coin).await?;
        let outcome = synchronize(&context.coin, vec![id_account as u32], current_height, fast).await?;
        synced_height(outcome, current_height)
    }

    async fn pay(id_account: i32, payment: Payment, context: &Context) -> FieldResult<String> {
        check_auth(context, id_account, true)?;
        let coin = &context.coin;
        let pczt = prepare_tx(id_account, payment, coin).await?;
        let mut connection = coin.get_connection().await?;
        let network = coin.network();
        let signed_pczt =
            crate::pay::sign::sign(&network, &mut connection, id_account as u32, pczt, &()).await?;
        let txid = crate::api::pay::broadcast(&signed_pczt, id_account as u32, coin).await?;
        Ok(txid)
    }

    /// Issue a new ZSA (Zcash Shielded Asset).
    ///
    /// Returns the transaction ID of the issuance transaction.
    async fn issue_asset(
        id_account: i32,
        asset_name: String,
        amount: String,
        first_issuance: bool,
        finalize: bool,
        context: &Context,
    ) -> FieldResult<String> {
        check_auth(context, id_account, true)?;
        let amount: u64 = amount
            .parse()
            .map_err(|_| "Invalid amount: must be a non-negative integer")?;
        let coin = &context.coin;
        let tx_bytes = crate::api::issuance::issue_asset(
            asset_name,
            amount,
            first_issuance,
            finalize,
            None,
            id_account as u32,
            coin,
        )
        .await?;
        let txid = crate::api::pay::broadcast_bytes(&tx_bytes, id_account as u32, coin).await?;
        Ok(txid)
    }

    /// Set or update the human-readable name of an asset.
    ///
    /// This is needed for assets discovered via blockchain scan (where the
    /// name is not transmitted on-chain). Also useful for renaming.
    async fn set_asset_name(id_asset: i32, name: String, context: &Context) -> FieldResult<bool> {
        check_admin_auth(context)?;
        let mut connection = context.coin.get_connection().await?;
        crate::sync::chain_log::set_asset_name(&mut connection, id_asset as i64, &name).await?;
        Ok(true)
    }

    async fn new_addresses(id_account: i32, context: &Context) -> FieldResult<Addresses> {
        check_auth(context, id_account, false)?;
        let network = context.coin.network();
        let mut conn = context.coin.get_connection().await?;
        generate_next_dindex(&network, &mut conn, id_account as u32).await?;
        let addresses =
            crate::account::get_addresses(&context.coin.network(), &mut conn, id_account as u32, 6)
                .await?;
        Ok(addresses.into())
    }

    /// `dkgVersion` defaults to the current generation (2, ZIP 2005); 0 or
    /// 1 form a legacy group, for tests of the upgrade path.
    pub async fn dkg_start(
        name: String,
        threshold: i32,
        participants: i32,
        message_account: i32,
        id_participant: i32,
        dkg_version: Option<i32>,
        context: &Context,
    ) -> FieldResult<String> {
        check_auth(context, message_account, true)?;
        crate::graphql::frost::dkg_start(
            name,
            threshold,
            participants,
            message_account,
            id_participant,
            dkg_version,
            context,
        )
        .await
    }

    /// Create (or return) the ZIP 2005 sibling of a legacy frost account.
    pub async fn frost_upgrade(id_account: i32, context: &Context) -> FieldResult<i32> {
        check_admin_auth(context)?;
        crate::graphql::frost::frost_upgrade(id_account, context).await
    }

    /// Plan the sweep of a legacy frost account into its sibling and start
    /// its signing session; returns the PCZT for the other signers.
    pub async fn frost_sweep(
        id_account: i32,
        id_coordinator: i32,
        message_account: i32,
        context: &Context,
    ) -> FieldResult<String> {
        check_auth(context, id_account, true)?;
        crate::graphql::frost::frost_sweep(id_account, id_coordinator, message_account, context)
            .await
    }

    pub async fn dkg_cancel(context: &Context) -> FieldResult<bool> {
        check_admin_auth(context)?;
        crate::graphql::frost::dkg_cancel(context).await
    }

    pub async fn dkg_set_address(
        id_participant: i32,
        address: String,
        context: &Context,
    ) -> FieldResult<bool> {
        check_admin_auth(context)?;
        crate::graphql::frost::dkg_set_address(id_participant, address, context).await
    }

    pub async fn do_dkg(context: &Context) -> FieldResult<bool> {
        check_admin_auth(context)?;
        crate::graphql::frost::do_dkg(context).await
    }

    /// The member's `zkfrost` backup string: as sensitive as a seed phrase.
    pub async fn frost_export_backup(id_account: i32, context: &Context) -> FieldResult<String> {
        check_auth(context, id_account, true)?;
        crate::graphql::frost::frost_export_backup(id_account, context).await
    }

    /// Restore frost accounts from a backup string and the group's UFVK:
    /// the legacy account, the ZIP 2005 one, or both; by default the one
    /// the key matches. `birth` defaults to Orchard activation.
    pub async fn frost_restore(
        name: String,
        backup: String,
        ufvk: String,
        legacy: Option<bool>,
        zip2005: Option<bool>,
        birth: Option<i32>,
        context: &Context,
    ) -> FieldResult<Vec<FrostRestoreResult>> {
        check_admin_auth(context)?;
        crate::graphql::frost::frost_restore(name, backup, ufvk, legacy, zip2005, birth, context)
            .await
    }

    pub async fn frost_sign(
        id_coordinator: i32,
        id_account: i32,
        message_account: i32,
        pczt: String,
        context: &Context,
    ) -> FieldResult<bool> {
        check_auth(context, id_account, true)?;
        crate::graphql::frost::frost_sign(
            id_coordinator,
            id_account,
            message_account,
            pczt,
            context,
        )
        .await
    }

    /// Run one step of the note migration process (split non-SD → SD, then
    /// migrate SD to Ironwood). Idempotent — call every ~6 seconds until
    /// Complete.
    async fn step_migration(id_account: i32, context: &Context) -> FieldResult<MigrationEvent> {
        check_auth(context, id_account, true)?;
        let event = crate::migrate::step::step_once(&context.coin, id_account as u32)
            .await
            .map_err(|e| format!("Migration error: {e}"))?;
        Ok(match event {
            crate::migrate::step::StepOutcome::Split { fee } => MigrationEvent {
                event: "SplitComplete".to_string(),
                fee: Some(fee as i32),
                message: None,
            },
            crate::migrate::step::StepOutcome::Migrated { fee } => MigrationEvent {
                event: "MigrateComplete".to_string(),
                fee: Some(fee as i32),
                message: None,
            },
            crate::migrate::step::StepOutcome::Complete => MigrationEvent {
                event: "Complete".to_string(),
                fee: None,
                message: None,
            },
            crate::migrate::step::StepOutcome::NothingToDo => MigrationEvent {
                event: "NothingToDo".to_string(),
                fee: None,
                message: None,
            },
        })
    }

    pub async fn frost_cancel(context: &Context) -> FieldResult<bool> {
        check_admin_auth(context)?;
        crate::graphql::frost::require_frost(context.frost_enabled)?;
        context.driver.cancel(crate::frost::driver::Cancel::Sign).await?;
        Ok(true)
    }
}

/// How long the daemon's mempool monitor waits after a failed pass before
/// it tries again, matching the app's listener (#1525).
pub const MEMPOOL_RETRY_PAUSE: Duration = Duration::from_secs(5);

/// The daemon's mempool monitor. `connect` yields the network client for
/// each pass, so a connect failure is retried like any other failure
/// instead of ending the task; the daemon passes `Coin::client`. `pause`
/// is the wait after a failed pass, [`MEMPOOL_RETRY_PAUSE`] for the daemon.
pub async fn run_mempool<C, CF>(context: Context, connect: C, pause: Duration) -> anyhow::Result<()>
where
    C: Fn() -> CF,
    CF: std::future::Future<Output = anyhow::Result<crate::Client>>,
{
    let coin = context.coin.clone();
    let network = &coin.network();
    loop {
        let pass = async {
            let mut conn = coin.get_connection().await?;
            let mut client = connect().await?;
            run_mempool_pass(network, &mut conn, &mut client).await
        };
        if let Err(error) = pass.await {
            tracing::error!("mempool monitor: {error:#}; retrying in {pause:?}");
            tokio::time::sleep(pause).await;
        }
    }
}

/// [`run_mempool`] with the wallet's own client and pace: what the daemon
/// spawns. The binary sees this crate from outside, where `Coin::client`
/// is not visible, so the wiring lives here.
pub async fn run_mempool_daemon(context: Context) -> anyhow::Result<()> {
    let coin = context.coin.clone();
    run_mempool(
        context,
        move || {
            let coin = coin.clone();
            async move { coin.client().await }
        },
        MEMPOOL_RETRY_PAUSE,
    )
    .await
}

/// One monitoring pass: fans the mempool stream out to the subscribers
/// until the stream ends or fails.
async fn run_mempool_pass(
    network: &crate::api::coin::Network,
    conn: &mut sqlx::SqliteConnection,
    client: &mut crate::Client,
) -> anyhow::Result<()> {
    {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<MempoolMsg>(10);
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    MempoolMsg::TxId(tx) => {
                        let txid = tx.txid;
                        let all_notes = tx.notes;
                        for item in tx.amounts {
                            let (account, value) = (item.account, item.value);
                            let notes: Vec<UnconfirmedNote> = all_notes
                                .iter()
                                .filter(|n| n.account == account)
                                .map(|n| UnconfirmedNote {
                                    pool: n.pool as i32,
                                    scope: n.scope as i32,
                                    value: zats_to_zec(n.value),
                                    diversifier: n
                                        .diversifier
                                        .as_deref()
                                        .map(hex::encode)
                                        .unwrap_or_default(),
                                    diversifier_index: n.diversifier_index.map(BigDecimal::from),
                                    address: n.address.clone(),
                                    memo: n.memo.clone(),
                                })
                                .collect();
                            {
                                let mut mempool = MEMPOOL.lock().await;
                                let e = mempool.unconfirmed.entry(account);
                                let e = e.or_insert_with(HashMap::new);
                                e.insert(txid.clone(), (zats_to_zec(value), notes.clone()));
                            }
                            {
                                let account = account as i32;
                                let mut ss = SUBS.lock().await;
                                if let Some(subs) = ss.get_mut(&account) {
                                    fan_out(
                                        subs,
                                        &Event {
                                            r#type: EventType::Tx,
                                            txid: txid.clone(),
                                            value: zats_to_zec(value),
                                            notes: notes.clone(),
                                            ..Event::default()
                                        },
                                    );
                                }
                            }
                        }
                    }
                    MempoolMsg::BlockHeight(height) => {
                        let mut ss = SUBS.lock().await;
                        let event = Event {
                            r#type: EventType::Block,
                            height: height as i32,
                            ..Event::default()
                        };
                        for subs in ss.values_mut() {
                            fan_out(subs, &event);
                        }
                    }
                }
            }
        });

        {
            let mut mempool = MEMPOOL.lock().await;
            mempool.unconfirmed.clear();
        }
        let cancel_token = CancellationToken::new();
        crate::mempool::run_mempool_impl(tx, network, conn, client, cancel_token).await
    }
}

#[derive(Default)]
pub struct Mempool {
    pub unconfirmed: HashMap<u32, HashMap<String, (BigDecimal, Vec<UnconfirmedNote>)>>,
}

pub static MEMPOOL: LazyLock<Mutex<Mempool>> = LazyLock::new(|| Mutex::new(Mempool::default()));

#[cfg(test)]
mod input_tests {
    use super::*;
    use crate::api::coin::Coin;
    use crate::graphql::query::Query;
    use juniper::{EmptySubscription, RootNode, Variables};

    const MNEMONIC: &str = "equal clock rain latin plastic toss scrub modify clarify fold armor exchange gesture erase habit plug state forward demise demand limb risk only document";

    async fn context() -> Context {
        let path = format!("/tmp/graphql_input_{}.db", uuid::Uuid::new_v4());
        let coin = Coin::new(Some(3)).open_database(path, None).await.unwrap();
        Context::new(coin)
    }

    /// Runs one mutation; the first error's message, or the JSON value.
    async fn mutate(context: &Context, mutation: &str) -> Result<serde_json::Value, String> {
        let schema = RootNode::new(Query {}, Mutation {}, EmptySubscription::<Context>::new());
        let (value, errors) =
            juniper::execute(&format!("mutation {{ {mutation} }}"), None, &schema, &Variables::new(), context)
                .await
                .map_err(|e| e.to_string())?;
        if let Some(error) = errors.first() {
            return Err(error.error().message().to_owned());
        }
        Ok(serde_json::to_value(value).unwrap())
    }

    fn create(aindex: i32, birth: i32) -> String {
        format!(
            "createAccount(newAccount: {{ name: \"a\", key: \"{MNEMONIC}\", aindex: {aindex}, birth: {birth}, useInternal: false }})"
        )
    }

    async fn accounts(context: &Context) -> Vec<(u32, u32)> {
        let mut conn = context.coin.get_connection().await.unwrap();
        sqlx::query_as("SELECT id_account, birth FROM accounts ORDER BY id_account")
            .fetch_all(&mut *conn)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn use_internal_may_be_left_out_and_follows_the_key() {
        const WIF: &str = "L4jNzRuAe1kHvGj7N5UXd3m9erYWRkV6EnXiHR6AoMM37dEzUkTV";
        let context = context().await;
        let omitted = |key: &str, name: &str| {
            format!("createAccount(newAccount: {{ name: \"{name}\", key: \"{key}\", aindex: 0, birth: 1 }})")
        };
        mutate(&context, &omitted(MNEMONIC, "seed")).await.unwrap();
        mutate(&context, &omitted(WIF, "wif")).await.unwrap();
        let mut conn = context.coin.get_connection().await.unwrap();
        let flags: Vec<(String, bool)> =
            sqlx::query_as("SELECT name, use_internal FROM accounts ORDER BY id_account")
                .fetch_all(&mut *conn)
                .await
                .unwrap();
        assert_eq!(flags, [("seed".to_string(), true), ("wif".to_string(), false)]);
        drop(conn);

        let explicit = format!(
            "createAccount(newAccount: {{ name: \"bad\", key: \"{WIF}\", aindex: 0, birth: 1, useInternal: true }})"
        );
        assert_eq!(
            mutate(&context, &explicit).await.unwrap_err(),
            "Use Internal Change must be off for this type of key"
        );
        assert_eq!(accounts(&context).await.len(), 2);
    }

    #[tokio::test]
    async fn editing_use_internal_resets_sync_only_when_it_changes() {
        const SYNCED: u32 = 1_000_000;
        let context = context().await;
        let id = mutate(&context, &format!(
            "createAccount(newAccount: {{ name: \"a\", key: \"{MNEMONIC}\", aindex: 0, birth: 1 }})"
        ))
        .await
        .unwrap()["createAccount"]
            .as_i64()
            .unwrap();
        let edit = |flag: bool| {
            format!("editAccount(idAccount: {id}, updateAccount: {{ useInternal: {flag} }})")
        };
        let state = || async {
            let mut conn = context.coin.get_connection().await.unwrap();
            let flag: (bool,) = sqlx::query_as("SELECT use_internal FROM accounts WHERE id_account = ?")
                .bind(id)
                .fetch_one(&mut *conn)
                .await
                .unwrap();
            let height: (u32,) = sqlx::query_as("SELECT MIN(height) FROM sync_heights WHERE account = ?")
                .bind(id)
                .fetch_one(&mut *conn)
                .await
                .unwrap();
            (flag.0, height.0)
        };
        let advance = || async {
            let mut conn = context.coin.get_connection().await.unwrap();
            sqlx::query("UPDATE sync_heights SET height = ? WHERE account = ?")
                .bind(SYNCED)
                .bind(id)
                .execute(&mut *conn)
                .await
                .unwrap();
        };

        advance().await;
        mutate(&context, &edit(true)).await.unwrap();
        assert_eq!(state().await, (true, SYNCED), "the same value keeps the sync data");
        mutate(&context, &format!("editAccount(idAccount: {id}, updateAccount: {{ name: \"b\" }})"))
            .await
            .unwrap();
        assert_eq!(state().await, (true, SYNCED), "leaving it out keeps the sync data");

        mutate(&context, &edit(false)).await.unwrap();
        let (flag, height) = state().await;
        assert!(!flag);
        assert!(height < SYNCED, "turning it off resets the sync data");

        advance().await;
        mutate(&context, &edit(true)).await.unwrap();
        let (flag, height) = state().await;
        assert!(flag);
        assert!(height < SYNCED, "turning it on resets the sync data");
    }

    #[tokio::test]
    async fn a_negative_birth_or_aindex_is_refused_before_anything_is_stored() {
        let context = context().await;
        assert_eq!(mutate(&context, &create(0, -2)).await.unwrap_err(), "Invalid birth");
        assert_eq!(mutate(&context, &create(-1, 10)).await.unwrap_err(), "Invalid aindex");
        assert!(accounts(&context).await.is_empty());
    }

    #[tokio::test]
    async fn an_edit_with_a_negative_birth_leaves_the_stored_birth_alone() {
        let context = context().await;
        let created = mutate(&context, &create(0, 10)).await.unwrap();
        let id = created["createAccount"].as_i64().unwrap();
        assert_eq!(accounts(&context).await, vec![(id as u32, 10)]);

        let edit = |birth: i32| format!("editAccount(idAccount: {id}, updateAccount: {{ birth: {birth} }})");
        assert_eq!(mutate(&context, &edit(-1)).await.unwrap_err(), "Invalid birth");
        assert_eq!(accounts(&context).await, vec![(id as u32, 10)]);

        assert_eq!(mutate(&context, &edit(20)).await.unwrap()["editAccount"], true);
        assert_eq!(accounts(&context).await, vec![(id as u32, 20)]);
    }
}

#[cfg(test)]
mod mempool_retry_tests {
    use super::*;
    use crate::api::coin::Coin;
    use std::sync::{Arc, Mutex as StdMutex};
    use tokio::time::Instant;

    /// A server whose every call fails.
    struct Down;

    #[tonic::async_trait]
    impl crate::net::LwdServer for Down {
        type CompactBlockStream =
            tokio_stream::wrappers::ReceiverStream<anyhow::Result<crate::lwd::CompactBlock>>;
        type TransactionStream = tokio_stream::wrappers::ReceiverStream<
            anyhow::Result<(u32, zcash_primitives::transaction::Transaction, usize)>,
        >;
        async fn latest_height(&mut self) -> anyhow::Result<u32> {
            anyhow::bail!("connection refused")
        }
        async fn block(&mut self, _: &crate::api::coin::Network, _: u32) -> anyhow::Result<crate::lwd::CompactBlock> {
            unimplemented!()
        }
        async fn block_range(&mut self, _: &crate::api::coin::Network, _: u32, _: u32) -> anyhow::Result<Self::CompactBlockStream> {
            unimplemented!()
        }
        async fn transaction(&mut self, _: &crate::api::coin::Network, _: &[u8]) -> anyhow::Result<(u32, zcash_primitives::transaction::Transaction)> {
            unimplemented!()
        }
        async fn post_transaction(&mut self, _: u32, _: &[u8]) -> anyhow::Result<String> {
            unimplemented!()
        }
        async fn taddress_txs(&mut self, _: &crate::api::coin::Network, _: &str, _: u32, _: u32) -> anyhow::Result<Self::TransactionStream> {
            unimplemented!()
        }
        async fn mempool_stream(&mut self, _: &crate::api::coin::Network) -> anyhow::Result<Self::TransactionStream> {
            unimplemented!()
        }
        async fn tree_state(&mut self, _: u32) -> anyhow::Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
            unimplemented!()
        }
    }

    async fn context() -> Context {
        let path = format!("/tmp/graphql_mempool_{}.db", uuid::Uuid::new_v4());
        let coin = Coin::new(Some(3)).open_database(path, None).await.unwrap();
        Context::new(coin)
    }

    /// A failing server is retried at the app's pace, and a failure to
    /// connect is retried the same way instead of ending the task (#1525).
    #[tokio::test]
    async fn a_failing_server_is_retried_at_the_pause_and_the_task_lives_on() {
        let context = context().await;
        let pause = Duration::from_millis(50);
        let attempts: Arc<StdMutex<Vec<Instant>>> = Arc::new(StdMutex::new(Vec::new()));
        let log = attempts.clone();
        let task = tokio::spawn(run_mempool(context, move || {
            let log = log.clone();
            async move {
                let n = {
                    let mut log = log.lock().unwrap();
                    log.push(Instant::now());
                    log.len()
                };
                // The first two attempts cannot even connect.
                if n <= 2 {
                    anyhow::bail!("no route to host");
                }
                Ok(Box::new(Down) as crate::Client)
            }
        }, pause));
        tokio::time::sleep(pause * 4 + pause / 2).await;
        assert!(!task.is_finished(), "the monitor does not end on an error");
        let attempts = attempts.lock().unwrap().clone();
        assert!(
            (4..=5).contains(&attempts.len()),
            "one attempt, then one per pause: {}",
            attempts.len()
        );
        for pair in attempts.windows(2) {
            assert!(pair[1] - pair[0] >= pause, "attempts are paced");
        }
        task.abort();
    }
}
