use anyhow::Result;
#[cfg(feature = "flutter")]
use sqlx::sqlite::SqliteRow;
use sqlx::{query, query_as, Row};
use tonic::{
    transport::{Channel, ClientTlsConfig},
};
pub use zcvlib::pod::{ElectionPropsPub, QuestionProp};
use zcvlib::vote_rpc::vote_streamer_client::VoteStreamerClient;
#[cfg(feature = "flutter")]
use zcvlib::{
    api::{
        simple::{import_account, import_election},
        ProgressReporter,
    },
};

pub use zcvlib::context::Context;

#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

#[cfg(feature = "flutter")]
use crate::frb_generated::StreamSink;

use crate::api::coin::Coin;

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb)]
pub async fn compile_election_def(election_json: String, seed: String) -> Result<String> {
    let election_def = zcvlib::api::simple::compile_election_def(election_json, seed)?;
    Ok(election_def)
}

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb(mirror(ElectionPropsPub)))]
pub struct _ElectionPropsPub {
    pub pir: String,
    pub end: u32,
    pub need_sig: bool,
    pub name: String,
    pub questions: Vec<QuestionProp>,
    pub caption: String,
    pub address: String,
    pub domain: Vec<u8>,
}

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb(mirror(QuestionProp)))]
pub struct _QuestionProp {
    pub title: String,
    pub subtitle: String,
    pub answers: Vec<String>,
}

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb)]
pub async fn parse_election(election_json: String) -> Result<ElectionPropsPub> {
    let e: ElectionPropsPub = serde_json::from_str(&election_json)?;
    Ok(e)
}

impl Coin {
    pub async fn to_context(&self) -> Result<Context> {
        let mut conn = self.get_connection().await?;
        let (url,): (String,) = query_as(
            "SELECT url FROM v_state
            WHERE id = 0",
        )
        .fetch_one(&mut *conn)
        .await?;
        let context = Context::new(&self.db_filepath, &self.url, &url).await?;
        Ok(context)
    }
}

// #[cfg(feature = "flutter")]
// #[cfg_attr(feature = "flutter", frb)]
// pub async fn set_election_urls(election_url: String, pir_url: String, c: &Coin) -> Result<()> {
//     let mut conn = c.get_connection().await?;
//     query(
//         "UPDATE v_state
//         SET election_url = ?1,
//         pir_url = ?2
//         WHERE id = 0",
//     )
//     .bind(&election_url)
//     .bind(&pir_url)
//     .execute(&mut *conn)
//     .await?;
//     Ok(())
// }

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb)]
pub async fn get_election(c: &Coin) -> Result<(u32, String, Option<ElectionPropsPub>)> {
    let mut conn = c.get_connection().await?;
    let (account, url) = query(
        "SELECT s.account, s.url FROM v_state s
        WHERE s.id = 0")
    .map(|r: SqliteRow| {
        let account: u32 = r.get(0);
        let url: String = r.get(1);
        (account, url)
    })
    .fetch_one(&mut *conn)
    .await?;
    let election = query(
        "SELECT e.data FROM v_elections e
        WHERE e.id_election = 0")
    .map(|r: SqliteRow| {
        let data: String = r.get(0);
        let election = serde_json::from_str::<ElectionPropsPub>(&data).unwrap();
        election
    })
    .fetch_optional(&mut *conn)
    .await?;
    Ok((account, url, election))
}

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb)]
pub async fn fetch_election(account: u32, url: String, c: &Coin) -> Result<ElectionPropsPub> {
    let c = c.to_context().await?;
    let (election, _, _) = import_election(account, &url, &c).await?;
    Ok(election)
}

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb)]
pub async fn import_election_account(account: u32, c: &Coin) -> Result<()> {
    let c = c.to_context().await?;
    import_account(account, &c).await?;
    Ok(())
}

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb)]
pub async fn delete_election(c: &Coin) -> Result<()> {
    tracing::info!("delete_election");
    let c = c.to_context().await?;
    zcvlib::api::simple::client_delete_election(&c).await?;
    Ok(())
}

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb)]
pub async fn delete_election_data(new_account: Option<u32>, c: &Coin) -> Result<()> {
    tracing::info!("delete_election_data");
    let c = c.to_context().await?;
    zcvlib::api::simple::client_delete_election_data(&c, new_account).await?;
    Ok(())
}

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb)]
pub async fn scan_ballots(id_account: u32, c: &Coin) -> Result<()> {
    let c = c.to_context().await?;
    tracing::info!("scan_ballots {}", c.election_url);
    zcvlib::api::simple::scan_ballots(id_account, &c).await?;
    Ok(())
}

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb)]
pub async fn get_balance(id_account: u32, c: &Coin) -> Result<u64> {
    tracing::info!("get_balance");
    let c = c.to_context().await?;
    let balance = zcvlib::api::simple::get_balance(id_account, &c).await?;
    Ok(balance)
}

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb)]
pub async fn vote(id_account: u32, vote: String, amount: u64, c: &Coin) -> Result<String> {
    tracing::info!("get_balance");
    let c = c.to_context().await?;
    let txid = zcvlib::api::simple::vote(id_account, vote, amount, &c).await?;
    Ok(hex::encode(&txid))
}

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb)]
pub async fn delegate(id_account: u32, address: String, amount: u64, c: &Coin) -> Result<String> {
    tracing::info!("delegate");
    let c = c.to_context().await?;
    let txid = zcvlib::api::simple::delegate(id_account, &address, amount, &c).await?;
    Ok(hex::encode(&txid))
}

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb)]
pub async fn get_election_address(id_account: u32, c: &Coin) -> Result<String> {
    tracing::info!("get_election_address");
    let c = c.to_context().await?;
    let address = zcvlib::api::simple::get_account_address(id_account, &c).await?;
    Ok(address)
}

#[cfg(feature = "flutter")]
impl ProgressReporter for StreamSink<u32> {
    fn report(&self, p: u32) {
        let _ = self.add(p);
    }
}
