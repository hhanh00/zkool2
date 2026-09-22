//! FRB API for authenticated voting configuration and the round list.

use std::{collections::HashMap, collections::HashSet, path::PathBuf};

use anyhow::{anyhow, ensure, Result};
#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;
use serde::{Deserialize, Serialize};

use crate::{api::coin::Coin, voting, voting::sidecar::VotingSidecar};

fn voting_network(c: &Coin) -> Result<zcash_voting::Network> {
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
    pub snapshot_height: Option<u64>,
    pub bundle_count: u32,
    pub action: String,
    pub proposals: Vec<VotingProposalListItem>,
}

pub(crate) async fn wallet_id(c: &Coin) -> Result<String> {
    let mut connection = c.get_connection().await?;
    let fingerprint = crate::db::get_account_fingerprint(&mut connection, c.account)
        .await?
        .ok_or_else(|| anyhow!("account {} has no seed fingerprint", c.account))?;
    ensure!(
        fingerprint.len() == 32,
        "seed fingerprint must be 32 bytes, got {}",
        fingerprint.len()
    );
    Ok(hex::encode(fingerprint))
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
pub async fn voting_round_list(c: &Coin) -> Result<Vec<VotingRoundListItem>> {
    let mut connection = c.get_connection().await?;
    let Some(source) = crate::db::get_prop(&mut connection, "voting_config_url")
        .await?
        .filter(|source| !source.is_empty())
    else {
        return Ok(Vec::new());
    };
    drop(connection);

    let client = crate::net::http::client(
        crate::net::http::proxy_url(c.transport, &c.proxy),
        std::time::Duration::from_secs(15),
    )?;
    let config = voting::net::voting_config_resolve(&source, &client).await?;
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
    let rounds: Vec<_> = voting::net::fetch_rounds(&servers, &client)
        .await?
        .into_iter()
        .filter(|round| authenticated_ids.contains(round.round_id.as_str()))
        .collect();
    let inputs: Vec<_> = rounds
        .iter()
        .map(|round| voting::summary::RoundInput {
            round_id: round.round_id.clone(),
            proposal_ids: round.proposals.iter().map(|proposal| proposal.id).collect(),
        })
        .collect();

    let sidecar =
        voting::sidecar::VotingSidecar::open(PathBuf::from(&c.db_filepath), wallet_id(c).await?)
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
                round_id: round.round_id,
                status,
                snapshot_height: summary.snapshot_height,
                bundle_count: summary.bundle_count,
                action: action.into(),
                proposals,
            })
        })
        .collect()
}

/// Parses a completed UI ballot as fork `DraftVote` JSON and stores its
/// durable choices in the wallet's voting sidecar. A skipped proposal is
/// encoded as `choice == num_options` by the UI.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_save_ballot(round_id: &str, drafts_json: &str, c: &Coin) -> Result<()> {
    let drafts = serde_json::from_str(drafts_json)?;
    let sidecar = VotingSidecar::open(PathBuf::from(&c.db_filepath), wallet_id(c).await?).await?;
    sidecar
        .save_ballot(round_id.to_owned(), voting_network(c)?, drafts)
        .await
}

#[cfg(test)]
mod tests {
    use super::normalized_status;

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
