//! FRB API for authenticated voting configuration and the round list.

use std::{collections::HashMap, collections::HashSet, path::PathBuf};

use anyhow::{anyhow, ensure, Result};
#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;
use serde::{Deserialize, Serialize};

use crate::{api::coin::Coin, net::http::Route, voting, voting::sidecar::VotingSidecar};

/// Cap on a voting response read over Tor; the chain transport uses the same.
const MAX_VOTING_RESPONSE_BYTES: usize = 8 << 20;

/// The route for the voting discovery GETs: config, round parameters and
/// the rounds overview. They go to the config host and the vote servers,
/// the same hosts that later receive the vote, so they must take the same
/// transport the vote takes (#1580).
pub(crate) fn voting_route(c: &Coin) -> Result<Route> {
    Ok(c.route(std::time::Duration::from_secs(15))?
        .capped(MAX_VOTING_RESPONSE_BYTES))
}

pub(crate) fn voting_network(c: &Coin) -> Result<zcash_voting::Network> {
    match c.coin {
        0 => Ok(zcash_voting::Network::Mainnet),
        1 => Ok(zcash_voting::Network::Testnet),
        2 => Ok(zcash_voting::Network::Regtest),
        3 => Err(anyhow!("voting is not supported on the ZSA network")),
        _ => Err(anyhow!("unsupported wallet network {}", c.coin)),
    }
}

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VotingProposalListItem {
    pub proposal_id: u32,
    pub title: String,
    pub options: Vec<String>,
}

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VotingRoundListItem {
    pub round_id: String,
    pub title: String,
    pub status: String,
    pub snapshot_height: u32,
    pub bundle_count: u32,
    pub action: String,
    pub proposals: Vec<VotingProposalListItem>,
    /// The helper fleet resolved for this listing, so opening a round can
    /// start share tracking without re-resolving the authenticated config.
    /// The vote servers double as the helper (share) servers.
    pub helper_urls: Vec<String>,
    /// Vote-end boundary as the overview reported it, passed straight to
    /// share tracking. `None` when the server omits it.
    pub vote_end_time: Option<u64>,
}

/// The id that scopes an account's rows in the voting sidecar.
///
/// Accounts derived from one seed at different ZIP-32 indices are separate
/// voters, each with its own notes and delegation identity, so the id is the
/// seed fingerprint and the account index together. The fingerprint alone
/// made two such accounts read and overwrite each other's ballots (#1567).
pub(crate) async fn wallet_id(account: u32, c: &Coin) -> Result<String> {
    let mut connection = c.get_connection().await?;
    let fingerprint = crate::db::get_account_fingerprint(&mut connection, account)
        .await?
        .ok_or_else(|| anyhow!("account {} has no seed fingerprint", account))?;
    let aindex = crate::db::get_account_aindex(&mut connection, account).await?;
    sidecar_wallet_id(&fingerprint, aindex)
}

fn sidecar_wallet_id(fingerprint: &[u8], aindex: u32) -> Result<String> {
    ensure!(
        fingerprint.len() == 32,
        "seed fingerprint must be 32 bytes, got {}",
        fingerprint.len()
    );
    Ok(format!("{}:{aindex}", hex::encode(fingerprint)))
}

fn normalized_status(status: &serde_json::Value) -> String {
    let status = status
        .as_str()
        .map(|status| status.trim().to_lowercase())
        .unwrap_or_else(|| status.to_string());
    match status.as_str() {
        "0" => "unspecified".into(),
        "1" => "active".into(),
        "2" => "tallying".into(),
        "3" => "finalized".into(),
        "4" => "pending".into(),
        "5" => "ceremony_failed".into(),
        _ => status,
    }
}

/// Fetches authenticated server rounds and combines them with sidecar state.
/// The returned action is display-only; execution must obtain a fresh resume
/// plan from the voting crate.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_round_list(account: u32, c: &Coin) -> Result<Vec<VotingRoundListItem>> {
    let mut connection = c.get_connection().await?;
    let Some(source) = crate::db::get_prop(&mut connection, "voting_config_url")
        .await?
        .filter(|source| !source.is_empty())
    else {
        return Ok(Vec::new());
    };
    drop(connection);

    let route = voting_route(c)?;
    let config = voting::net::voting_config_resolve(&source, &route).await?;
    let servers: Vec<_> = config
        .vote_servers
        .iter()
        .map(|server| server.url.clone())
        .collect();
    let authenticated_ids: HashSet<_> = config
        .authenticated_rounds
        .iter()
        .map(|round| round.round_id.as_str())
        .collect();
    let rounds: Vec<_> = voting::net::fetch_rounds(&servers, &route)
        .await?
        .into_iter()
        .filter(|round| authenticated_ids.contains(round.round_id.as_str()))
        .collect();
    let mut snapshot_heights = HashMap::with_capacity(rounds.len());
    for round in &rounds {
        let params = voting::net::fetch_round_params(&round.round_id, &config, &route).await?;
        snapshot_heights.insert(
            round.round_id.clone(),
            u32::try_from(params.snapshot_height).map_err(|_| {
                anyhow!(
                    "snapshot height {} is out of range for round {}",
                    params.snapshot_height,
                    round.round_id
                )
            })?,
        );
    }
    let inputs: Vec<_> = rounds
        .iter()
        .map(|round| voting::summary::RoundInput {
            round_id: round.round_id.clone(),
            proposal_ids: round.proposals.iter().map(|proposal| proposal.id).collect(),
        })
        .collect();

    let sidecar =
        voting::sidecar::VotingSidecar::open(PathBuf::from(&c.db_filepath), wallet_id(account, c).await?)
            .await?;
    let local: HashMap<_, _> = voting::summary::load_local_summaries(&sidecar, &inputs)
        .await?
        .into_iter()
        .map(|summary| (summary.round_id.clone(), summary))
        .collect();

    rounds
        .into_iter()
        .map(|round| {
            let summary = local
                .get(&round.round_id)
                .ok_or_else(|| anyhow!("missing local summary for round {}", round.round_id))?;
            let status = normalized_status(&round.status);
            let finished = matches!(status.as_str(), "tallying" | "finalized");
            let action = match summary.action(finished) {
                voting::summary::ListAction::StartVoting => "start_voting",
                voting::summary::ListAction::Resume => "resume",
                voting::summary::ListAction::Review => "review",
                voting::summary::ListAction::ViewResults => "view_results",
            };
            let proposals = round
                .proposals
                .iter()
                .map(|proposal| VotingProposalListItem {
                    proposal_id: proposal.id,
                    title: proposal
                        .title
                        .clone()
                        .filter(|title| !title.trim().is_empty())
                        .unwrap_or_else(|| format!("Proposal {}", proposal.id)),
                    options: if proposal.options.is_empty() {
                        vec!["Yes".into(), "No".into()]
                    } else {
                        proposal
                            .options
                            .iter()
                            .enumerate()
                            .map(|(index, option)| {
                                option
                                    .display_label()
                                    .unwrap_or_else(|| format!("Option {}", index + 1))
                            })
                            .collect()
                    },
                })
                .collect();
            Ok(VotingRoundListItem {
                title: round.display_title(),
                snapshot_height: *snapshot_heights.get(&round.round_id).ok_or_else(|| {
                    anyhow!(
                        "missing authenticated snapshot height for round {}",
                        round.round_id
                    )
                })?,
                round_id: round.round_id,
                status,
                bundle_count: summary.bundle_count,
                action: action.into(),
                proposals,
                helper_urls: servers.clone(),
                vote_end_time: round.vote_end_time,
            })
        })
        .collect()
}

/// FRB representation of the sidecar's ballot decision.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Choice { choice: u32 },
    Skipped,
}

impl From<zcash_voting::session::Decision> for Decision {
    fn from(decision: zcash_voting::session::Decision) -> Self {
        match decision {
            zcash_voting::session::Decision::Choice(choice) => Self::Choice { choice },
            zcash_voting::session::Decision::Skipped => Self::Skipped,
        }
    }
}

impl From<Decision> for zcash_voting::session::Decision {
    fn from(decision: Decision) -> Self {
        match decision {
            Decision::Choice { choice } => Self::Choice(choice),
            Decision::Skipped => Self::Skipped,
        }
    }
}

/// A proposal and its saved decision. Missing entries are unanswered.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug)]
pub struct VotingSelection {
    pub proposal_id: u32,
    pub decision: Decision,
}

/// Loads selections from the wallet's voting sidecar before submission.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_load_selections(
    round_id: &str,
    account: u32,
    c: &Coin,
) -> Result<Vec<VotingSelection>> {
    let sidecar = VotingSidecar::open(PathBuf::from(&c.db_filepath), wallet_id(account, c).await?).await?;
    let round_id = round_id.to_owned();
    sidecar
        .run(move |db| {
            Ok(db
                .ballot_intents(&round_id)?
                .into_iter()
                .map(|(proposal_id, decision)| VotingSelection {
                    proposal_id,
                    decision: decision.into(),
                })
                .collect())
        })
        .await
}

/// Saves a selection or skip without submitting a vote.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_save_selection(
    round_id: &str,
    proposal_id: u32,
    decision: Decision,
    num_options: u32,
    account: u32,
    c: &Coin,
) -> Result<()> {
    let sidecar = VotingSidecar::open(PathBuf::from(&c.db_filepath), wallet_id(account, c).await?).await?;
    ensure_ballot_round(&sidecar, round_id, c).await?;
    let network = voting_network(c)?;
    let round_id = round_id.to_owned();
    sidecar
        .run(move |db| {
            db.set_ballot_intents(
                &round_id,
                network,
                &[(proposal_id, decision.into(), num_options)],
            )
        })
        .await
}

/// Ballot intents reference a round row, even before delegation is prepared.
/// Once initialized, editing and reloading selections needs no network.
async fn ensure_ballot_round(sidecar: &VotingSidecar, round_id: &str, c: &Coin) -> Result<()> {
    let id = round_id.to_owned();
    if sidecar.run(move |db| Ok(db.round(&id)?.is_some())).await? {
        return Ok(());
    }
    let mut connection = c.get_connection().await?;
    let source = crate::db::get_prop(&mut connection, "voting_config_url")
        .await?
        .filter(|source| !source.is_empty())
        .ok_or_else(|| anyhow!("voting configuration is missing"))?;
    drop(connection);
    let route = voting_route(c)?;
    let config = voting::net::voting_config_resolve(&source, &route).await?;
    let params = voting::net::fetch_round_params(round_id, &config, &route).await?;
    let network = voting_network(c)?;
    sidecar
        .run(move |db| db.ensure_round(network, &params, None))
        .await
}

/// Returns a proposal to unanswered, subject to the voting lifecycle guards.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_clear_selection(
    round_id: &str,
    proposal_id: u32,
    account: u32,
    c: &Coin,
) -> Result<()> {
    let sidecar = VotingSidecar::open(PathBuf::from(&c.db_filepath), wallet_id(account, c).await?).await?;
    let round_id = round_id.to_owned();
    sidecar
        .run(move |db| db.clear_ballot_intent(&round_id, proposal_id))
        .await
}

#[cfg(test)]
mod tests {
    use super::{normalized_status, sidecar_wallet_id};

    #[test]
    fn sidecar_id_separates_accounts_of_one_seed() {
        let fingerprint = [0xabu8; 32];
        let first = sidecar_wallet_id(&fingerprint, 0).unwrap();
        let seventh = sidecar_wallet_id(&fingerprint, 7).unwrap();
        assert_eq!(first, format!("{}:0", "ab".repeat(32)));
        assert_eq!(seventh, format!("{}:7", "ab".repeat(32)));
        assert_ne!(first, seventh);
    }

    #[test]
    fn sidecar_id_rejects_a_short_fingerprint() {
        assert!(sidecar_wallet_id(&[0u8; 16], 0).is_err());
    }

    #[test]
    fn normalizes_numeric_and_named_statuses() {
        assert_eq!(normalized_status(&serde_json::json!(1)), "active");
        assert_eq!(
            normalized_status(&serde_json::json!(" FINALIZED ")),
            "finalized"
        );
        assert_eq!(normalized_status(&serde_json::json!(5)), "ceremony_failed");
    }
}
