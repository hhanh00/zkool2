use anyhow::Result;
use sqlx::{query, query_as};
use tonic::{
    transport::{Channel, ClientTlsConfig},
    Request,
};
#[cfg(feature = "flutter")]
use zcvlib::{api::ProgressReporter, db::store_election};
pub use zcvlib::pod::{ChoiceProp, ElectionPropsPub, QuestionPropPub};
use zcvlib::vote_rpc::{vote_streamer_client::VoteStreamerClient, Hash};

pub use zcvlib::context::Context;

#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

#[cfg(feature = "flutter")]
use crate::frb_generated::StreamSink;

use crate::{api::coin::Coin};

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
    pub questions: Vec<QuestionPropPub>,
}

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb(mirror(QuestionPropPub)))]
pub struct _QuestionPropPub {
    pub title: String,
    pub subtitle: String,
    pub index: usize,
    pub address: String,
    pub choices: Vec<ChoiceProp>,
}

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb(mirror(ChoiceProp)))]
pub struct _ChoiceProp {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub answers: Vec<String>,
}

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb)]
pub async fn parse_election(election_json: String) -> Result<ElectionPropsPub> {
    let e: ElectionPropsPub = serde_json::from_str(&election_json)?;
    Ok(e)
}

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
pub struct ElectionId {
    pub url: Option<String>,
    pub hash: Vec<u8>,
}

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb)]
pub async fn get_election_context(c: &Coin) -> Result<Context> {
    let mut conn = c.get_connection().await?;
    let (election_url,): (String,) = query_as(
        "SELECT url FROM v_state
        WHERE id = 0",
    )
    .fetch_one(&mut *conn)
    .await?;
    tracing::info!("get_election_context");
    let context = Context::new(&c.db_filepath, &c.url, &election_url).await?;
    Ok(context)
}

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb)]
pub async fn get_election_id(c: &Context) -> Result<ElectionId> {
    let mut conn = c.connect().await?;
    let (hash, url): (Vec<u8>, Option<String>) = query_as(
        "SELECT hash, url FROM v_state
        WHERE id = 0",
    )
    .fetch_one(&mut *conn)
    .await?;
    Ok(ElectionId { url, hash })
}

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb)]
pub async fn get_election(c: &Context) -> Result<ElectionPropsPub> {
    let mut conn = c.connect().await?;
    let (data,): (String,) = query_as(
        "SELECT e.data FROM v_elections e
        JOIN v_state s ON s.hash = e.hash
        WHERE s.id = 0",
    )
    .fetch_one(&mut *conn)
    .await?;
    let e: ElectionPropsPub = serde_json::from_str(&data)?;
    Ok(e)
}

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb)]
pub async fn fetch_election(url: String, hash: Vec<u8>, c: &Context) -> Result<ElectionPropsPub> {
    let mut conn = c.connect().await?;
    let mut client = connect_voted(url.clone()).await?;
    let election_json = client
        .get_election(Request::new(Hash { hash: hash.clone() }))
        .await?
        .into_inner()
        .election;
    query("UPDATE v_state SET hash = ?1, url = ?2 WHERE id = 0")
        .bind(&hash)
        .bind(&url)
        .execute(&mut *conn)
        .await?;
    let election: ElectionPropsPub = serde_json::from_str(&election_json)?;
    store_election(&mut *conn, &election).await?;
    Ok(election)
}

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb)]
pub async fn delete_election(c: &Context) -> Result<()> {
    tracing::info!("delete_election");
    zcvlib::api::simple::client_delete_election(&c).await?;
    Ok(())
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

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb)]
pub async fn scan_votes(progress: StreamSink<u32>, hash: String, id_account: u32, c: &Context) -> Result<()> {
    tracing::info!("scan_votes");
    zcvlib::api::simple::scan_notes(hash, id_account, &progress, &c).await?;
    Ok(())
}

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb)]
pub async fn scan_ballots(hash: String, id_account: u32, c: &Context) -> Result<()> {
    tracing::info!("scan_ballots");
    zcvlib::api::simple::scan_ballots(hash, vec![id_account], &c).await?;
    Ok(())
}

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb)]
pub async fn get_balance(hash: String, id_account: u32, idx_question: u32, c: &Context) -> Result<u64> {
    tracing::info!("get_balance");
    let balance = zcvlib::api::simple::get_balance(hash, id_account, idx_question, &c).await?;
    Ok(balance)
}

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb)]
pub async fn vote(hash: String, id_account: u32, idx_question: u32, vote: String, amount: u64, c: &Context) -> Result<()> {
    tracing::info!("get_balance");
    zcvlib::api::simple::vote(hash, id_account, idx_question, vote, amount, &c).await?;
    Ok(())
}

#[cfg(feature = "flutter")]
impl ProgressReporter for StreamSink<u32> {
    fn report(&self, p: u32) {
        let _ = self.add(p);
    }
}
