//! zkool's assembly of the voting crate's round executor and driver.
//!
//! The crate's [`RoundExecutor`] owns the whole cast-vote lifecycle for one
//! bound round: from the persisted ballot intents it plans Delegate /
//! CastVote / AdvanceVote steps, proves, persists, and advances chain
//! submission through the host's transport. zkool supplies only transports,
//! keys, and per-dispatch context; this module builds those adapters and the
//! process-local run registry that drives the executor to quiescence.

use std::sync::Arc;

use anyhow::{ensure, Result};
use zeroize::Zeroizing;
use zcash_voting::prelude::{VotingDb, VotingHotkey};
use zcash_voting::vote_work::{ProposalRosterEntry, RoundBinding, RoundExecutor};
use zcash_voting::{ChainSubmissionClientConfig, HelperClient, HelperHealth, Network};

use crate::voting::chain::{ZkoolChainTransport, ZkoolTreeTransport};
use crate::voting::sidecar::ZkoolHelperTransport;

/// Transports and endpoints one executor run needs.
#[derive(Clone, Debug)]
pub struct ExecutorRoutes {
    /// zkool transport selector: 0 direct, 1 Tor, 2 Nym, 3 proxy.
    pub transport: u8,
    /// External proxy URL, used when `transport` is 3.
    pub proxy: String,
    /// Ordered vote-chain base URLs; POST failover and status lookup follow
    /// this order.
    pub chain_endpoints: Vec<String>,
}

/// Builds a round executor for `binding` over zkool's routed transports.
///
/// The executor freezes the wallet scope of `db` at construction, so the
/// returned executor cannot be retargeted at another wallet. Helper share
/// delivery rides the same routes as chain submission.
pub fn build_executor(
    db: Arc<VotingDb>,
    binding: RoundBinding,
    routes: &ExecutorRoutes,
) -> Result<RoundExecutor<ZkoolChainTransport>> {
    ensure!(
        !routes.chain_endpoints.is_empty(),
        "round driving requires at least one vote-chain endpoint"
    );
    let chain_transport = ZkoolChainTransport::new(routes.transport, &routes.proxy);
    let chain_config =
        ChainSubmissionClientConfig::for_network(binding.network, routes.chain_endpoints.clone());
    let helper_client = HelperClient::new(
        Arc::new(ZkoolHelperTransport::new(
            routes.transport,
            &routes.proxy,
        )),
        HelperHealth::default(),
    );
    let tree_transport = Arc::new(ZkoolTreeTransport::new(
        routes.transport,
        &routes.proxy,
    ));
    let executor = RoundExecutor::with_transport(db, chain_transport, chain_config, helper_client)?
        .with_tree_transport(tree_transport)
        .with_binding(binding)?;
    Ok(executor)
}

/// Builds the immutable per-round binding: round, roster, and hotkey.
///
/// `proposals` are `(proposal_id, num_options)` pairs from the authenticated
/// round configuration. The hotkey is bound so the executor can reconstruct
/// the signer on the proving thread; without it the executor refuses to plan
/// `CastVote`.
pub fn build_binding(
    round_id: &str,
    network: Network,
    proposals: &[(u32, u32)],
    hotkey: Option<&VotingHotkey>,
) -> Result<RoundBinding> {
    ensure!(!proposals.is_empty(), "round binding requires a roster");
    Ok(RoundBinding {
        round_id: round_id.to_owned(),
        network,
        proposals: proposals
            .iter()
            .map(|(proposal_id, num_options)| ProposalRosterEntry {
                proposal_id: *proposal_id,
                num_options: *num_options,
            })
            .collect(),
        hotkey_secret: hotkey.map(|hotkey| Zeroizing::new(hotkey.stored_secret().to_vec())),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binding_rejects_an_empty_roster() {
        assert!(build_binding(&"01".repeat(32), Network::Regtest, &[], None).is_err());
    }

    #[test]
    fn binding_maps_the_roster_and_hotkey() {
        let hotkey = zcash_voting::prelude::generate_random_voting_hotkey(Network::Regtest)
            .expect("hotkey");
        let binding = build_binding(
            &"01".repeat(32),
            Network::Regtest,
            &[(1, 2), (3, 4)],
            Some(&hotkey),
        )
        .expect("binding");
        assert_eq!(binding.proposals.len(), 2);
        assert_eq!(binding.proposals[1].num_options, 4);
        assert_eq!(
            binding.hotkey_secret.expect("secret").as_slice(),
            hotkey.stored_secret()
        );
    }
}
