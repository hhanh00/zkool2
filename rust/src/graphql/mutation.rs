use std::{collections::HashMap, sync::LazyLock};

use crate::{
    account::generate_next_dindex,
    api::{mempool::MempoolMsg, pay::DustChangePolicy},
    graphql::{
        data::{Addresses, Event, EventType},
        query::{zats_to_zec, zec_to_zats},
        subs::SUBS,
        Context,
    },
    pay::plan::{extract_transaction, sign_transaction},
    Sink,
};
use bigdecimal::BigDecimal;
use juniper::{graphql_object, FieldResult, GraphQLInputObject};
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

    async fn synchronize(id_accounts: Vec<i32>, context: &Context) -> FieldResult<bool> {
        let id_accounts = id_accounts.into_iter().map(|v| v as u32).collect();
        let current_height = crate::api::network::get_current_height(&context.coin).await?;
        crate::sync::synchronize_impl(
            (),
            id_accounts,
            current_height,
            100_000,
            40,
            10_000,
            &context.coin,
        )
        .await?;
        Ok(true)
    }

    async fn pay(id_account: i32, payment: Payment, context: &Context) -> FieldResult<String> {
        let height = crate::api::network::get_current_height(&context.coin).await?;
        let mut recipients = vec![];
        for r in payment.recipients {
            recipients.push(crate::pay::Recipient {
                address: r.address,
                amount: zec_to_zats(r.amount)? as u64,
                pools: None,
                user_memo: r.memo,
                memo_bytes: None,
                price: None,
            });
        }
        let network = context.coin.network();
        let mut connection = context.coin.get_connection().await?;
        let mut client = context.coin.client().await?;

        let pczt = crate::pay::plan::plan_transaction(
            &network,
            &mut connection,
            &mut client,
            id_account as u32,
            payment.src_pools.unwrap_or(7) as u8,
            &recipients,
            false,
            payment.recipient_pays_fee.unwrap_or_default(),
            DustChangePolicy::Discard,
            None,
        )
        .await?;
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
            crate::account::get_addresses(&context.coin.network(), &mut conn, id_account as u32, 7)
                .await?;
        let addresses = Addresses {
            ua: addresses.ua,
            transparent: addresses.taddr,
            sapling: addresses.saddr,
            orchard: addresses.oaddr,
        };
        Ok(addresses)
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
