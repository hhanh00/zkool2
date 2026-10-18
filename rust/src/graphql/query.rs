use anyhow::{Error, Result};
use dataloader::cached::Loader;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;

use crate::api::coin::{Coin, Network};
use crate::api::pay::PcztPackage;
use crate::db::{calculate_balance, get_sync_height};
use crate::graphql::data::{
    Account, Addresses, AssetInfo, Balance, FrostParamsResult, Note, Transaction, UnconfirmedTx,
};
use crate::graphql::mutation::MEMPOOL;
use crate::graphql::mutation::{Output, Payment, UnsignedTx};
use crate::graphql::{check_admin_auth, check_auth, Context};
use crate::pay::{
    pool::ALL_POOLS,
    TxPlan,
};

use bigdecimal::num_bigint::BigInt;
use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive};
use chrono::NaiveDateTime;
use juniper::{graphql_object, FieldError, FieldResult, GraphQLInputObject};

pub struct Query {}

#[derive(GraphQLInputObject, Default)]
pub struct AccountFilter {
    id: Option<i32>,
    name: Option<String>,
}

#[graphql_object]
#[graphql(context = Context)]
impl Query {
    /// Saved swaps for the specified account, newest first. No provider requests.
    async fn near_intents_list_swaps(
        id_account: i32,
        pending_only: Option<bool>,
        context: &Context,
    ) -> FieldResult<Vec<crate::near_intents::SavedSwap>> {
        if id_account <= 0 {
            return Err(juniper::FieldError::new(
                "Invalid account ID",
                juniper::Value::Null,
            ));
        }
        check_auth(context, id_account, false)?;
        Ok(crate::api::near_intents::near_intents_list_swaps(
            pending_only.unwrap_or(false),
            id_account as u32,
            &context.coin,
        )
        .await?)
    }

    async fn near_intents_assets(
        context: &Context,
    ) -> FieldResult<Vec<crate::near_intents::SwapAsset>> {
        check_admin_auth(context)?;
        Ok(crate::api::near_intents::near_intents_assets(&context.coin).await?)
    }

    async fn near_intents_status(
        deposit_address: String,
        deposit_memo: Option<String>,
        context: &Context,
    ) -> FieldResult<crate::near_intents::SwapStatus> {
        check_admin_auth(context)?;
        Ok(crate::api::near_intents::near_intents_status(
            deposit_address,
            deposit_memo,
            &context.coin,
        )
        .await?)
    }

    fn api_version() -> &'static str {
        "1.0"
    }

    /// Lists authenticated voting rounds for the specified account.
    async fn voting_rounds(
        id_account: i32,
        context: &Context,
    ) -> FieldResult<Vec<crate::graphql::voting::VotingRound>> {
        crate::graphql::voting::rounds(id_account, context).await
    }

    /// Returns one authenticated round, including its proposal roster.
    async fn voting_round(
        id_account: i32,
        round_id: String,
        context: &Context,
    ) -> FieldResult<crate::graphql::voting::VotingRound> {
        crate::graphql::voting::round(id_account, round_id, context).await
    }

    /// Polls asynchronous vote and helper-share submission progress.
    async fn voting_submission_status(
        id_account: i32,
        round_id: String,
        context: &Context,
    ) -> FieldResult<crate::graphql::voting::VotingSubmissionStatus> {
        crate::graphql::voting::submission_status(id_account, round_id, context).await
    }

    async fn accounts(
        account_filter: Option<AccountFilter>,
        context: &Context,
    ) -> FieldResult<Vec<Account>> {
        check_admin_auth(context)?;
        let mut conn = context.coin.get_connection().await?;
        let accounts = crate::db::list_accounts(&mut conn, context.coin.coin).await?;
        let mut accounts: Vec<_> = accounts
            .into_iter()
            .map(|a| Account {
                id: a.id as i32,
                name: a.name,
                seed: a.seed,
                passphrase: a.passphrase,
                aindex: a.aindex as i32,
                dindex: a.dindex as i32,
                birth: a.birth as i32,
                height: a.height as i32,
                balance: zats_to_zec(a.balance as i64),
                incoming_only: a.incoming_only,
            })
            .collect();

        let account_filter = account_filter.unwrap_or_default();
        if let Some(id) = account_filter.id {
            accounts.retain(|a| a.id == id);
        };
        if let Some(name) = account_filter.name {
            accounts.retain(|a| a.name == name);
        }

        Ok(accounts)
    }

    pub async fn transactions_by_account(
        id_account: i32,
        height: Option<i32>,
        context: &Context,
    ) -> FieldResult<Vec<Transaction>> {
        check_auth(context, id_account, false)?;
        let min_height = u32::try_from(height.unwrap_or_default()).unwrap_or_default();
        let mut conn = context.coin.get_connection().await?;
        let transactions =
            crate::db::fetch_txs_from(&mut conn, id_account as u32, min_height).await?;
        Ok(transactions.into_iter().map(Into::into).collect())
    }

    pub async fn transaction_by_id(
        id_account: i32,
        txid: String,
        context: &Context,
    ) -> FieldResult<Transaction> {
        check_auth(context, id_account, false)?;
        let mut txid = hex::decode(&txid)?;
        txid.reverse();
        let mut conn = context.coin.get_connection().await?;
        let transaction = crate::db::fetch_tx_by_txid(&mut conn, id_account as u32, &txid)
            .await?
            .ok_or(FieldError::new("Unknown txid", juniper::Value::Null))?;
        Ok(transaction.into())
    }

    pub async fn memos_by_transaction(
        id_account: i32,
        id_transaction: i32,
        context: &Context,
    ) -> FieldResult<Vec<String>> {
        check_auth(context, id_account, false)?;
        let mut conn = context.coin.get_connection().await?;
        let memos =
            crate::account::tx_memos(&mut conn, id_account as u32, id_transaction as u32).await?;
        Ok(memos.into_iter().filter_map(|m| m.memo).collect())
    }

    pub async fn balance_by_account(
        id_account: i32,
        height: Option<i32>,
        context: &Context,
    ) -> FieldResult<Balance> {
        check_auth(context, id_account, false)?;
        let height = height.map(|h| h as u32);
        let mut conn = context.coin.get_connection().await?;
        let current_height = get_sync_height(&mut conn, id_account as u32).await?;
        let height = height.or(current_height);
        let b = calculate_balance(&mut conn, id_account as u32, height).await?;
        let total = b.0[0] + b.0[1] + b.0[2] + b.0[3];
        let balance = Balance {
            height: height.map(|h| h as i32),
            transparent: zats_to_zec(b.0[0] as i64),
            sapling: zats_to_zec(b.0[1] as i64),
            orchard: zats_to_zec(b.0[2] as i64),
            ironwood: zats_to_zec(b.0[3] as i64),
            total: zats_to_zec(total as i64),
        };
        Ok(balance)
    }

    /// A frost account's parameters, DKG version, sibling and recoverability;
    /// null for an account that is not a frost account.
    async fn frost_params(
        id_account: i32,
        context: &Context,
    ) -> FieldResult<Option<FrostParamsResult>> {
        check_auth(context, id_account, false)?;
        crate::graphql::frost::frost_params(id_account, context).await
    }

    /// Export viewing keys for the selected pools (defaults to all available pools).
    /// Transparent-only selections return an xpub. Ironwood shares Orchard's key.
    /// With `incoming`, the incoming viewing key instead: it shows payments
    /// received and nothing else; transparent alone gives the xpub of the
    /// external chain (depth 4, child 0).
    async fn unified_full_viewing_key(
        id_account: i32,
        pools: Option<i32>,
        incoming: Option<bool>,
        context: &Context,
    ) -> FieldResult<String> {
        check_auth(context, id_account, false)?;
        let account = u32::try_from(id_account)?;
        let available = crate::api::account::get_account_pools(account, &context.coin).await?;
        let pools = pools.unwrap_or(i32::from(available));
        if pools <= 0 || pools & !i32::from(ALL_POOLS) != 0 {
            return Err(FieldError::new(
                "Invalid pool mask: expected 1 through 15",
                juniper::Value::Null,
            ));
        }
        let pools = pools as u8;
        if pools & !available != 0 {
            return Err(FieldError::new(
                "Selected pools are unavailable for this account",
                juniper::Value::Null,
            ));
        }
        if incoming.unwrap_or(false) {
            Ok(crate::api::account::get_account_uivk(account, pools, &context.coin).await?)
        } else {
            Ok(crate::api::account::get_account_ufvk(account, pools, &context.coin).await?)
        }
    }

    async fn address_by_account(
        id_account: i32,
        pools: Option<i32>,
        context: &Context,
    ) -> FieldResult<Addresses> {
        check_auth(context, id_account, false)?;
        let mut conn = context.coin.get_connection().await?;
        let ua_pools = pools.unwrap_or(6) as u8;
        let addresses = crate::account::get_addresses(
            &context.coin.network(),
            &mut conn,
            id_account as u32,
            ua_pools,
        )
        .await?;
        Ok(addresses.into())
    }

    async fn unconfirmed_by_account(
        id_account: i32,
        context: &Context,
    ) -> FieldResult<Vec<UnconfirmedTx>> {
        check_auth(context, id_account, false)?;
        let mempool = MEMPOOL.lock().await;
        if let Some(unconfirmed_txs) = mempool.unconfirmed.get(&(id_account as u32)) {
            let txs: Vec<_> = unconfirmed_txs
                .iter()
                .map(|(txid, (value, notes))| UnconfirmedTx {
                    txid: txid.clone(),
                    value: value.clone(),
                    notes: notes.clone(),
                })
                .collect();
            return Ok(txs);
        }
        Ok(vec![])
    }

    async fn notes_by_account(
        id_account: i32,
        diversifier_indices: Option<Vec<BigDecimal>>,
        context: &Context,
    ) -> FieldResult<Vec<Note>> {
        check_auth(context, id_account, false)?;
        let network = context.coin.network();
        let mut conn = context.coin.get_connection().await?;
        let keys = crate::db::account_viewing_keys(&mut conn, id_account as u32).await?;

        let indices: Option<Vec<i64>> = diversifier_indices
            .map(|list| list.into_iter().filter_map(|bd| bd.to_i64()).collect());

        let notes = crate::db::get_notes(&mut conn, id_account as u32).await?;
        let notes: Vec<_> = notes
            .into_iter()
            .filter(|n| match &indices {
                None => true,
                Some(indices) => n
                    .diversifier_index
                    .as_ref()
                    .is_some_and(|di| indices.contains(di)),
            })
            .map(|n| resolve_note(&network, &keys, n))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to resolve note: {e}"))?;
        Ok(notes)
    }

    /// List all known assets with per-account unspent balances.
    async fn assets(id_account: i32, context: &Context) -> FieldResult<Vec<AssetInfo>> {
        check_auth(context, id_account, false)?;
        let mut conn = context.coin.get_connection().await?;
        let rows = crate::db::get_zsa_holdings(&mut conn, id_account as u32)
            .await
            .map_err(|e| format!("Failed to query assets: {e}"))?;

        let assets = rows
            .into_iter()
            .map(|r| {
                let is_zec = r.asset_base == [0u8; 32];
                AssetInfo {
                    id_asset: r.id_asset as i32,
                    asset_desc_hash: hex::encode(&r.asset_desc_hash),
                    asset_name: r.asset_name,
                    ik: hex::encode(&r.ik),
                    asset_base: hex::encode(&r.asset_base),
                    finalized: r.finalized,
                    first_seen_height: r.first_seen_height,
                    balance: if is_zec {
                        zats_to_zec(r.balance)
                    } else {
                        BigDecimal::from(r.balance)
                    },
                }
            })
            .collect();

        Ok(assets)
    }

    async fn current_height(context: &Context) -> FieldResult<i32> {
        let height = crate::api::network::get_current_height(&context.coin).await?;
        Ok(height as i32)
    }

    async fn prepare_send(
        id_account: i32,
        payment: Payment,
        context: &Context,
    ) -> FieldResult<String> {
        check_auth(context, id_account, false)?;
        let tx = prepare_tx(id_account, payment, &context.coin).await?;
        let txbin = bincode::encode_to_vec(&tx, bincode::config::standard())?;
        let txhex = hex::encode(&txbin);
        Ok(txhex)
    }

    async fn decode_pczt(pczt: String, context: &Context) -> FieldResult<UnsignedTx> {
        let pczt = hex::decode(&pczt)?;
        let (pczt, _) =
            bincode::decode_from_slice::<PcztPackage, _>(&pczt, bincode::config::standard())?;
        let tx_plan = TxPlan::from_package(&context.coin.network(), &pczt)?;
        let unsigned = tx_plan.to_unsigned_tx();
        Ok(unsigned)
    }

    async fn sign_tx(id_account: i32, pczt: String, context: &Context) -> FieldResult<String> {
        check_auth(context, id_account, true)?;
        let mut connection = context.coin.get_connection().await?;
        let pczt = hex::decode(&pczt)?;
        let (pczt, _) =
            bincode::decode_from_slice::<PcztPackage, _>(&pczt, bincode::config::standard())?;
        let network = context.coin.network();
        let signed =
            crate::pay::sign::sign(&network, &mut connection, id_account as u32, pczt, &()).await?;
        let tx_bin = crate::pay::plan::extract_transaction(&signed).await?;
        let tx = hex::encode(&tx_bin);
        Ok(tx)
    }

    async fn broadcast_tx(tx_bytes: String, context: &Context) -> FieldResult<String> {
        let tx = hex::decode(&tx_bytes)?;
        let mut client = context.coin.client().await?;
        let height = client.latest_height().await?;
        let result = client.post_transaction(height, &tx).await?;
        Ok(result)
    }
}

impl TxPlan {
    pub fn to_unsigned_tx(self) -> UnsignedTx {
        let recipients: Vec<_> = self
            .outputs
            .into_iter()
            .enumerate()
            .map(|(id, o)| Output {
                id: 0,
                pool: o.pool as i32,
                vout: id as i32,
                address: o.address,
                value: zats_to_zec(o.amount as i64),
                memo: None,
            })
            .collect();
        let fee = zats_to_zec(self.fee as i64);
        UnsignedTx { recipients, fee }
    }
}

pub async fn prepare_tx(
    id_account: i32,
    payment: Payment,
    coin: &Coin,
) -> FieldResult<crate::api::pay::PcztPackage> {
    let mut connection = coin.get_connection().await?;
    let mut client = coin.client().await?;

    let mut recipients = vec![];
    for r in payment.recipients {
        let (asset_base, asset_name) =
            crate::pay::resolve_asset(&mut connection, r.asset_desc.as_deref()).await?;

        let amount: u64 = if asset_base == [0u8; 32] {
            zec_to_zats(r.amount)? as u64
        } else {
            r.amount
                .to_string()
                .parse::<u64>()
                .map_err(|e| format!("Invalid ZSA amount: {e}"))?
        };

        recipients.push(crate::pay::Recipient {
            address: r.address,
            amount,
            pools: None,
            user_memo: r.memo,
            memo_bytes: None,
            price: None,
            asset_base,
            asset_name,
        });
    }
    let network = coin.network();

    let request = crate::pay::plan::PaymentRequest {
        recipients,
        src_pools: payment.src_pools.unwrap_or(ALL_POOLS.into()) as u8,
        recipient_pays_fee: payment.recipient_pays_fee.unwrap_or_default(),
        confirmations: payment.confirmations.map(|v| v as u32).unwrap_or_default(),
        category: None,
    };
    let pczt = crate::pay::plan::plan_transaction(
        &network,
        &mut connection,
        &mut client,
        id_account as u32,
        &request,
        &crate::pay::plan::PlanMode::Standard,
    )
    .await?;
    Ok(pczt)
}

fn resolve_note(
    network: &Network,
    keys: &crate::keys::AccountViewingKeys,
    n: crate::api::account::TxNote,
) -> Result<Note, String> {
    let resolved = match (&n.diversifier, n.pool) {
        (None, 0) => None,
        (None, _) => return Err("shielded note missing diversifier".to_string()),
        (Some(diversifier), pool) => keys
            .note_address(network, pool, n.scope, diversifier)
            .map_err(|e| e.to_string())?,
    };
    let (address, diversifier_index) = match resolved {
        Some(r) => (Some(r.address), r.diversifier_index),
        None => (None, None),
    };

    let asset_base = if (n.pool == 2 || n.pool == 3) && n.id_asset.is_some() {
        Some("".to_string()) // Placeholder: could look up actual asset_base from assets
    } else {
        None
    };

    Ok(Note {
        id: n.id as i32,
        height: n.height as i32,
        pool: n.pool as i32,
        tx: n.tx as i32,
        scope: n.scope as i32,
        diversifier: n.diversifier.map(|d| hex::encode(&d)).unwrap_or_default(),
        diversifier_index: diversifier_index.map(BigDecimal::from),
        address: address.unwrap_or_default(),
        value: zats_to_zec(n.value as i64),
        memo: n.memo,
        id_asset: n.id_asset.map(|v| v as i32),
        asset_base,
    })
}

#[graphql_object]
impl Account {
    pub fn id(&self) -> i32 {
        self.id
    }
    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn seed(&self) -> Option<String> {
        self.seed.clone()
    }
    pub fn passphrase(&self) -> Option<String> {
        self.passphrase.clone()
    }
    pub fn aindex(&self) -> i32 {
        self.aindex
    }
    pub fn dindex(&self) -> i32 {
        self.dindex
    }
    pub fn birth(&self) -> i32 {
        self.birth
    }
    pub fn height(&self) -> i32 {
        self.height
    }
    pub fn balance(&self) -> BigDecimal {
        self.balance.clone()
    }
    /// Created from an incoming viewing key: `balance` is the total received,
    /// spends being invisible to it, and it cannot spend.
    pub fn incoming_only(&self) -> bool {
        self.incoming_only
    }

    pub async fn transactions(&self, context: &Context) -> FieldResult<Vec<Transaction>> {
        let txs = Query::transactions_by_account(self.id, None, context).await?;
        Ok(txs)
    }
}

#[graphql_object]
impl Transaction {
    pub fn id(&self) -> i32 {
        self.id
    }
    pub fn txid(&self) -> String {
        self.txid.clone()
    }
    pub fn account(&self) -> i32 {
        self.account
    }
    pub fn height(&self) -> i32 {
        self.height
    }
    pub fn time(&self) -> NaiveDateTime {
        self.time
    }
    pub fn value(&self) -> BigDecimal {
        self.value.clone()
    }
    pub fn fee(&self) -> BigDecimal {
        self.fee.clone()
    }

    pub async fn notes(&self, context: &Context) -> FieldResult<Vec<Note>> {
        let network = context.coin.network();
        let mut conn = context.coin.get_connection().await?;
        let keys = crate::db::account_viewing_keys(&mut conn, self.account as u32).await?;
        let notes = crate::db::get_tx_notes(&mut conn, self.account as u32, self.id as u32).await?;
        let notes: Vec<_> = notes
            .into_iter()
            .map(|n| resolve_note(&network, &keys, n))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to resolve note: {e}"))?;
        Ok(notes)
    }

    pub async fn outputs(&self, context: &Context) -> FieldResult<Vec<Output>> {
        let mut conn = context.coin.get_connection().await?;
        let outputs =
            crate::account::tx_outputs(&mut conn, self.account as u32, self.id as u32).await?;
        Ok(outputs
            .into_iter()
            .map(|o| Output {
                id: o.id as i32,
                pool: o.pool as i32,
                vout: o.vout as i32,
                value: zats_to_zec(o.value as i64),
                address: o.address,
                memo: o.memo,
            })
            .collect())
    }

    pub async fn spends(&self, context: &Context) -> FieldResult<Vec<Note>> {
        let network = context.coin.network();
        let mut conn = context.coin.get_connection().await?;
        let keys = crate::db::account_viewing_keys(&mut conn, self.account as u32).await?;
        let spends =
            crate::db::get_spent_notes(&mut conn, self.account as u32, self.id as u32).await?;
        let spends: Vec<_> = spends
            .into_iter()
            .map(|n| resolve_note(&network, &keys, n))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to resolve note: {e}"))?;
        Ok(spends)
    }
}

#[graphql_object]
impl Note {
    pub fn id(&self) -> i32 {
        self.id
    }
    pub fn height(&self) -> i32 {
        self.height
    }
    pub fn pool(&self) -> i32 {
        self.pool
    }
    pub fn value(&self) -> BigDecimal {
        self.value.clone()
    }
    pub fn address(&self) -> String {
        self.address.clone()
    }
    pub fn scope(&self) -> i32 {
        self.scope
    }
    pub fn diversifier(&self) -> String {
        self.diversifier.clone()
    }
    pub fn diversifier_index(&self) -> Option<BigDecimal> {
        self.diversifier_index.clone()
    }
    pub fn memo(&self) -> Option<String> {
        self.memo.clone()
    }
    pub async fn tx(&self, context: &Context) -> FieldResult<Transaction> {
        context
            .tx_loader
            .try_load(self.tx)
            .await
            .map_err(|_| anyhow::anyhow!("No tx exists for ID {}", self.tx))?
            .map_err(|err| FieldError::new(err.to_string(), juniper::Value::Null))
    }
}

pub fn zats_to_zec(zats: i64) -> bigdecimal::BigDecimal {
    let digits = BigInt::from_i64(zats).unwrap();
    BigDecimal::from_bigint(digits, 8)
}

pub fn zec_to_zats(zec: bigdecimal::BigDecimal) -> FieldResult<i64> {
    let (digit, _) = zec.with_scale(8).into_bigint_and_scale();
    let digits = digit.to_u64_digits().1;
    if digits.len() > 1 {
        return Err("Invalid amount".into());
    }
    Ok(digits[0] as i64)
}

pub struct TxDataLoader {}

pub struct TxBatcher {
    pub pool: SqlitePool,
}

impl dataloader::BatchFn<i32, Result<Transaction, Arc<Error>>> for TxBatcher {
    async fn load(&mut self, keys: &[i32]) -> HashMap<i32, Result<Transaction, Arc<Error>>> {
        let ids: Vec<u32> = keys
            .iter()
            .filter_map(|id| u32::try_from(*id).ok())
            .collect();
        let loaded = async {
            let mut conn = self.pool.acquire().await?;
            crate::db::fetch_txs_by_ids(&mut conn, &ids).await
        }
        .await;
        match loaded {
            Ok(txs) => txs
                .into_iter()
                .map(|tx| (tx.id as i32, Ok(Transaction::from(tx))))
                .collect(),
            // A failed batch fails every key in it rather than panicking the
            // request; the resolver surfaces the error per field.
            Err(error) => {
                let error = Arc::new(error);
                keys.iter()
                    .map(|id| (*id, Err(Arc::clone(&error))))
                    .collect()
            }
        }
    }
}

pub type TxLoader = Loader<i32, Result<Transaction, Arc<anyhow::Error>>, TxBatcher>;

pub fn new_tx_loader(pool: SqlitePool) -> TxLoader {
    TxLoader::new(TxBatcher { pool }).with_yield_count(100)
}
