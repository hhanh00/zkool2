use anyhow::Result;
#[cfg(feature = "flutter")]
use sqlx::sqlite::SqliteRow;
use sqlx::{query, query_as, Row};
use tonic::{
    transport::{Channel, ClientTlsConfig},
    Request,
};
pub use zcvlib::pod::{ElectionPropsPub, QuestionProp};
use zcvlib::vote_rpc::vote_streamer_client::VoteStreamerClient;
#[cfg(feature = "flutter")]
use zcvlib::{api::ProgressReporter, db::store_election, vote_rpc::Empty};

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
    pub start: u32,
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
        let (election_url,): (String,) = query_as(
            "SELECT url FROM v_state
        WHERE id = 0",
        )
        .fetch_one(&mut *conn)
        .await?;
        let context = Context::new(&self.db_filepath, &self.url, &election_url).await?;
        Ok(context)
    }
}

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb)]
pub async fn get_election_url(c: &Coin) -> Result<(Option<String>, Option<u32>)> {
    let mut conn = c.get_connection().await?;
    let (url, account) = query(
        "SELECT url, account FROM v_state
        WHERE id = 0",
    )
    .map(|r: SqliteRow| {
        let url: Option<String> = r.get(0);
        let account: Option<u32> = r.get(1);
        (url, account)
    })
    .fetch_one(&mut *conn)
    .await?;
    Ok((url, account))
}

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb)]
pub async fn get_election(c: &Coin) -> Result<ElectionPropsPub> {
    let mut conn = c.get_connection().await?;
    let data = query(
        "SELECT data FROM v_elections
        WHERE id_election = 0",
    )
    .map(|r: SqliteRow| r.get::<String, _>(0))
    .fetch_one(&mut *conn)
    .await?;
    let e = serde_json::from_str::<ElectionPropsPub>(&data)?;
    Ok(e)
}

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb)]
pub async fn fetch_election(url: String, account: u32, c: &Coin) -> Result<ElectionPropsPub> {
    let mut conn = c.get_connection().await?;
    let mut client = connect_voted(url.clone()).await?;
    let election_json = client
        .get_election(Request::new(Empty {}))
        .await?
        .into_inner()
        .election;
    query("UPDATE v_state SET url = ?1, account = ?2 WHERE id = 0")
        .bind(&url)
        .bind(account)
        .execute(&mut *conn)
        .await?;
    let election: ElectionPropsPub = serde_json::from_str(&election_json)?;
    store_election(&mut *conn, &election).await?;
    Ok(election)
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
pub async fn scan_votes(progress: StreamSink<u32>, id_account: u32, c: &Coin) -> Result<()> {
    let c = c.to_context().await?;
    tracing::info!("scan_votes {}", c.election_url);
    zcvlib::api::simple::scan_notes(vec![id_account], &progress, &c).await?;
    Ok(())
}

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb)]
pub async fn scan_ballots(id_account: u32, c: &Coin) -> Result<()> {
    let c = c.to_context().await?;
    tracing::info!("scan_ballots {}", c.election_url);
    zcvlib::api::simple::scan_ballots(vec![id_account], &c).await?;
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

async fn connect_voted(url: String) -> Result<VoteStreamerClient<Channel>> {
    let mut channel = tonic::transport::Channel::from_shared(url.clone())?;
    if url.starts_with("https") {
        let tls = ClientTlsConfig::new().with_enabled_roots();
        channel = channel.tls_config(tls)?;
    }
    let client = VoteStreamerClient::connect(channel).await?;
    Ok(client)
}
