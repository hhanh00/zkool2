use std::{collections::HashMap, sync::LazyLock};

use crate::{
    Sink, account::generate_next_dindex, api::{mempool::MempoolMsg}, graphql::{
        Context, data::{Addresses, Event, EventType}, query::{prepare_tx, zats_to_zec}, subs::SUBS
    }, pay::plan::{extract_transaction, sign_transaction}
};
use bigdecimal::BigDecimal;
use juniper::{graphql_object, FieldResult, GraphQLObject, GraphQLInputObject};
use tokio::sync::{mpsc::Sender, Mutex};
use tokio_util::sync::CancellationToken;

pub struct Mutation {}

#[derive(GraphQLInputObject)]
pub struct NewAccount {
    pub name: String,
    pub key: String,
    pub passphrase: Option<String>,
    pub aindex: i32,
    pub birth: Option<i32>,
    pub pools: Option<i32>,
    pub use_internal: bool,
}

#[derive(GraphQLInputObject)]
pub struct UpdateAccount {
    pub name: Option<String>,
    pub birth: Option<i32>,
}

#[derive(GraphQLInputObject)]
pub struct Recipient {
    pub address: String,
    pub amount: BigDecimal,
    pub memo: Option<String>,
}

#[derive(GraphQLInputObject)]
pub struct Payment {
    pub recipients: Vec<Recipient>,
    pub src_pools: Option<i32>,
    pub recipient_pays_fee: Option<bool>,
}

#[derive(GraphQLObject)]
pub struct Output {
    pub address: String,
    pub amount: BigDecimal,
}

#[derive(GraphQLObject)]
pub struct UnsignedTx {
    pub recipients: Vec<Output>,
    pub fee: BigDecimal,
}


#[graphql_object]
#[graphql(
    context = Context,
)]
impl Mutation {
    async fn create_account(new_account: NewAccount, context: &Context) -> FieldResult<i32> {
        let height = crate::api::network::get_current_height(&context.coin).await?;
        let na = crate::api::account::NewAccount {
            name: new_account.name,
            restore: false,
            key: new_account.key,
            passphrase: new_account.passphrase,
            fingerprint: None,
            icon: None,
            aindex: new_account.aindex as u32,
            birth: new_account.birth.map(|v| v as u32).or(Some(height)),
            pools: new_account.pools.map(|v| v as u8),
            use_internal: new_account.use_internal,
            folder: String::new(),
            internal: false,
            ledger: false,
        };
        let id_account = crate::api::account::new_account(&na, &context.coin).await?;
        Ok(id_account as i32)
    }

    async fn edit_account(
        id_account: i32,
        update_account: UpdateAccount,
        context: &Context,
    ) -> FieldResult<bool> {
        let ua = crate::api::account::AccountUpdate {
            coin: 0,
            id: id_account as u32,
            name: update_account.name,
            icon: None,
            birth: update_account.birth.map(|v| v as u32),
            folder: 0,
            hidden: None,
            enabled: None,
        };
        crate::api::account::update_account(&ua, &context.coin).await?;
        Ok(true)
    }

    async fn delete_account(id_account: i32, context: &Context) -> FieldResult<bool> {
        crate::api::account::delete_account(id_account as u32, &context.coin).await?;
        Ok(true)
    }

    async fn reset_account(id_account: i32, context: &Context) -> FieldResult<bool> {
        crate::api::account::reset_sync(id_account as u32, &context.coin).await?;
        Ok(true)
    }

    async fn synchronize(id_accounts: Vec<i32>, context: &Context) -> FieldResult<i32> {
        let id_accounts = id_accounts.into_iter().map(|v| v as u32).collect();
        let current_height = crate::api::network::get_current_height(&context.coin).await?;
        let height = crate::sync::synchronize_impl(
            (),
            id_accounts,
            current_height,
            100_000,
            40,
            10_000,
            &context.coin,
        )
        .await?;
        Ok(height as i32)
    }

    async fn pay(id_account: i32, payment: Payment, context: &Context) -> FieldResult<String> {
        let coin = &context.coin;
        let pczt = prepare_tx(id_account, payment, coin).await?;
        let mut connection = coin.get_connection().await?;
        let mut client = coin.client().await?;
        let height = client.latest_height().await?;
        let signed_pczt = sign_transaction(&mut connection, id_account as u32, &pczt).await?;
        let tx_bytes = extract_transaction(&signed_pczt).await?;
        let txid = crate::pay::send(&mut client, height, &tx_bytes).await?;
        Ok(txid)
    }

    async fn new_addresses(id_account: i32, context: &Context) -> FieldResult<Addresses> {
        let network = context.coin.network();
        let mut conn = context.coin.get_connection().await?;
        generate_next_dindex(&network, &mut conn, id_account as u32).await?;
        let addresses =
            crate::account::get_addresses(&context.coin.network(), &mut conn, id_account as u32, 6)
                .await?;
        let addresses = Addresses {
            ua: addresses.ua,
            transparent: addresses.taddr,
            sapling: addresses.saddr,
            orchard: addresses.oaddr,
        };
        Ok(addresses)
    }

    pub async fn dkg_start(
        name: String,
        threshold: i32,
        participants: i32,
        message_account: i32,
        id_participant: i32,
        context: &Context,
    ) -> FieldResult<String> {
        crate::graphql::frost::dkg_start(
            name,
            threshold,
            participants,
            message_account,
            id_participant,
            context,
        )
        .await
    }

    pub async fn dkg_cancel(context: &Context) -> FieldResult<bool> {
        crate::graphql::frost::dkg_cancel(context).await
    }

    pub async fn dkg_set_address(id_participant: i32, address: String, context: &Context) -> FieldResult<bool> {
        crate::graphql::frost::dkg_set_address(id_participant, address, context).await
    }

    pub async fn do_dkg(context: &Context) -> FieldResult<bool> {
        crate::graphql::frost::do_dkg(context).await
    }

    pub async fn frost_sign(id_coordinator: i32, id_account: i32, message_account: i32, pczt: String, context: &Context) -> FieldResult<bool> {
        crate::graphql::frost::frost_sign(id_coordinator, id_account, message_account, pczt, context).await
    }

    pub async fn frost_cancel(context: &Context) -> FieldResult<bool> {
        let mut connection = context.coin.get_connection().await?;
        crate::frost::dkg::delete_frost_state(&mut connection).await?;
        Ok(true)
    }
}

impl<T: Send + Sync> Sink<T> for Sender<T> {
    async fn send(&self, value: T) {
        let _ = tokio::sync::mpsc::Sender::send(self, value).await;
    }

    async fn send_error(&self, e: anyhow::Error) {
        tracing::error!("Error: {e}");
    }
}

pub async fn run_mempool(context: Context) -> anyhow::Result<()> {
    let coin = context.coin.clone();
    let network = &coin.network();
    loop {
        let coin = context.coin.clone();
        let mut conn = coin.get_connection().await?;
        let mut client = coin.client().await?;
        let (tx, mut rx) = tokio::sync::mpsc::channel::<MempoolMsg>(10);
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    MempoolMsg::TxId(txid, items, _) => {
                        for (account, _, value) in items {
                            {
                                let mut mempool = MEMPOOL.lock().await;
                                let e = mempool.unconfirmed.entry(account);
                                let e = e.or_insert_with(HashMap::new);
                                e.insert(txid.clone(), zats_to_zec(value));
                            }
                            {
                                let account = account as i32;
                                let ss = SUBS.lock().await;
                                if let Some(subs) = ss.get(&account) {
                                    for s in subs {
                                        let _ = s
                                            .send(Ok(Event {
                                                r#type: EventType::Tx,
                                                txid: txid.clone(),
                                                ..Event::default()
                                            }))
                                            .await;
                                    }
                                }
                            }
                        }
                    }
                    MempoolMsg::BlockHeight(height) => {
                        let ss = SUBS.lock().await;
                        for subs in ss.values() {
                            for s in subs {
                                let _ = s
                                    .send(Ok(Event {
                                        r#type: EventType::Block,
                                        height: height as i32,
                                        ..Event::default()
                                    }))
                                    .await;
                            }
                        }
                        let _ = crate::graphql::frost::new_block(coin.clone()).await;
                    }
                }
            }
        });

        let runner = async move {
            {
                let mut mempool = MEMPOOL.lock().await;
                mempool.unconfirmed.clear();
            }
            let cancel_token = CancellationToken::new();

            crate::mempool::run_mempool_impl(tx, network, &mut conn, &mut client, cancel_token)
                .await?;
            Ok::<_, anyhow::Error>(())
        };
        match runner.await {
            Ok(_) => {}
            Err(error) => {
                tracing::error!("Error: {error}");
            }
        }
    }
}

#[derive(Default)]
pub struct Mempool {
    pub unconfirmed: HashMap<u32, HashMap<String, BigDecimal>>,
}

pub static MEMPOOL: LazyLock<Mutex<Mempool>> = LazyLock::new(|| Mutex::new(Mempool::default()));
