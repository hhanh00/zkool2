use crate::{
    account::generate_next_dindex,
    api::pay::DustChangePolicy,
    graphql::{Context, data::Addresses, query::zec_to_zats},
    pay::plan::{extract_transaction, sign_transaction},
};
use bigdecimal::BigDecimal;
use juniper::{graphql_object, FieldResult, GraphQLInputObject};

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
