use anyhow::Result;
use zcvlib::{api::simple::{self}};
pub use zcvlib::pod::{ElectionPropsPub, QuestionPropPub, ChoiceProp};

#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb)]
pub async fn compile_election_def(election_json: String, seed: String) -> Result<String> {
    let election_def = simple::compile_election_def(election_json, seed)?;
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
