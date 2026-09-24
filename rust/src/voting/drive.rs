//! zkool's assembly of the voting crate's round executor and driver.
//!
//! The crate's [`RoundExecutor`] owns the whole cast-vote lifecycle for one
//! bound round: from the persisted ballot intents it plans Delegate /
//! CastVote / AdvanceVote steps, proves, persists, and advances chain
//! submission through the host's transport. zkool supplies only transports,
//! keys, and per-dispatch context; this module builds those adapters and the
//! process-local run registry that drives the executor to quiescence.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{ensure, Result};
use halo2_proofs::pasta::group::ff::PrimeField as _;
use rand_core::OsRng;
use zeroize::Zeroizing;
use zcash_keys::keys::UnifiedSpendingKey;
use zip32::AccountId;
use zcash_voting::prelude::{VotingDb, VotingHotkey};
use zcash_voting::vote_work::{
    DelegationStepInputs, ProposalRosterEntry, RoundBinding, RoundExecutor, RoundHostContext,
};
use zcash_voting::{
    delegation_pipeline::{
        DelegationAccountIdentity, DelegationPipeline, DelegationSigner,
    },
    ChainAdvancePolicy, ChainSubmissionClientConfig, HelperClient, HelperHealth, Network,
    PirFleet, RoundHostSource,
};

use crate::voting::chain::{ZkoolChainTransport, ZkoolPirTransport, ZkoolTreeTransport};
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

/// Signs a delegation signing request with the wallet's own ZIP-32 seed.
///
/// Mirrors the fork's wallet-example signer: verifies the request fingerprint
/// against the seed, derives the account SpendAuth key, randomizes it with the
/// request alpha, and signs the PCZT sighash.
fn sign_delegation_request(
    seed: &[u8],
    request: zcash_voting::prelude::DelegationSigningRequest,
) -> Result<[u8; 64], zcash_voting::VotingError> {
    use zcash_voting::VotingError;
    let seed_fingerprint = zip32::fingerprint::SeedFingerprint::from_seed(seed).ok_or_else(
        || VotingError::InvalidInput {
            message: "wallet seed length is not valid for ZIP-32".to_string(),
        },
    )?;
    if seed_fingerprint.to_bytes() != request.seed_fingerprint {
        return Err(VotingError::InvalidInput {
            message: "wallet seed fingerprint does not match delegation signing request"
                .to_string(),
        });
    }
    let account = AccountId::try_from(request.account_index).map_err(|_| {
        VotingError::InvalidInput {
            message: format!("invalid account_index {}", request.account_index),
        }
    })?;
    let usk = UnifiedSpendingKey::from_seed(&request.network, seed, account).map_err(
        |error| VotingError::InvalidInput {
            message: format!("account spending key derivation failed: {error}"),
        },
    )?;
    let sk = *usk.orchard();
    let ask = orchard::keys::SpendAuthorizingKey::from(&sk);
    let alpha = Option::<halo2_proofs::pasta::pallas::Scalar>::from(
        halo2_proofs::pasta::pallas::Scalar::from_repr(request.alpha),
    )
    .ok_or_else(|| VotingError::InvalidInput {
        message: "delegation alpha is not a valid Pallas scalar".to_string(),
    })?;
    let rsk = ask.randomize(&alpha);
    let sig = rsk.sign(&mut OsRng, &request.sighash);
    Ok((&sig).into())
}

/// Everything the host needs to gather before building delegation inputs.
pub struct DelegationHostInputs {
    /// Lightwalletd-derived round inputs (anchor tree state, branch ids).
    pub lwd: zcash_voting::delegate::DelegationLwdInputs,
    /// Orchard FVK bytes and ZIP-32 identity of the note-owning account.
    pub identity: DelegationAccountIdentity,
    /// Notes and snapshot witnesses loaded from the wallet.
    pub note_source: crate::voting::note_source::ZkoolNoteSource,
    /// Authenticated round session JSON, when the config supplies one.
    pub session_json: Option<String>,
    /// Note bundling policy for the round's bundles.
    pub bundle_policy: zcash_voting::prelude::BundlePolicy,
    /// Ordered PIR server URLs from the authenticated config.
    pub pir_endpoints: Vec<String>,
    /// PIR layout from the authenticated config.
    pub pir_layout: zcash_voting::config::PirLayout,
    /// ZIP-32 seed bytes of the note-owning wallet.
    pub seed: Vec<u8>,
}

/// Assembles the per-dispatch delegation inputs: pipeline driver, software
/// signer, and PIR fleet over zkool's routed transport.
pub fn build_delegation_step_inputs(
    db: Arc<VotingDb>,
    hotkey: &VotingHotkey,
    inputs: DelegationHostInputs,
    routes: &ExecutorRoutes,
) -> Result<DelegationStepInputs> {
    ensure!(
        !inputs.pir_endpoints.is_empty(),
        "delegation requires at least one PIR endpoint"
    );
    let pir = PirFleet::new(
        &inputs.pir_endpoints,
        inputs.pir_layout,
        Arc::new(ZkoolPirTransport::new(routes.transport, &routes.proxy)),
    )?;
    let hotkey = VotingHotkey::from_stored_secret(hotkey.stored_secret(), hotkey.network())
        .map_err(|error| anyhow::anyhow!("voting hotkey reload failed: {error}"))?;
    let pipeline = DelegationPipeline::new(
        db,
        inputs.note_source,
        inputs.lwd,
        inputs.identity,
        Some(hotkey),
        inputs.bundle_policy,
        inputs.session_json.as_deref(),
    )?;
    let seed = inputs.seed;
    let signer = DelegationSigner::Software(Arc::new(move |request| {
        sign_delegation_request(&seed, request)
    }));
    Ok(DelegationStepInputs {
        driver: Arc::new(pipeline),
        signer,
        pir: Arc::new(pir),
    })
}

/// Per-dispatch host inputs for one round's driver run.
///
/// The voting crate reads the context once per dispatch so a long run plans
/// against the clock it actually runs under; the delegation inputs are fixed
/// per run and cloned into each context.
pub struct ZkoolRoundHost {
    helper_urls: Vec<String>,
    vote_tree_node_urls: Vec<String>,
    ceremony_start: Option<u64>,
    vote_end: Option<u64>,
    delegation: Option<DelegationStepInputs>,
}

impl ZkoolRoundHost {
    pub fn new(
        helper_urls: Vec<String>,
        vote_tree_node_urls: Vec<String>,
        ceremony_start: Option<u64>,
        vote_end: Option<u64>,
        delegation: Option<DelegationStepInputs>,
    ) -> Self {
        Self {
            helper_urls,
            vote_tree_node_urls,
            ceremony_start,
            vote_end,
            delegation,
        }
    }
}

impl RoundHostSource for ZkoolRoundHost {
    fn host_context(&self) -> RoundHostContext {
        RoundHostContext {
            configured_helper_urls: self.helper_urls.clone(),
            now_seconds: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            ceremony_start_seconds: self.ceremony_start,
            vote_end_time_seconds: self.vote_end,
            vote_tree_node_urls: self.vote_tree_node_urls.clone(),
            delegation: self.delegation.clone(),
            // Fresh submissions start status-only; persisted work always
            // starts with exact-tree recovery inside the client.
            chain_policy: ChainAdvancePolicy::default(),
            // Proofs run on a dedicated big-stack thread via the crate's
            // proving runtime; one at a time keeps memory bounded on phones.
            max_proof_concurrency: 1,
        }
    }
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

    #[test]
    fn signer_rejects_a_foreign_fingerprint_and_signs_its_own() {
        let seed = vec![7u8; 32];
        let fingerprint = zip32::fingerprint::SeedFingerprint::from_seed(&seed)
            .expect("fingerprint")
            .to_bytes();
        let request = |seed_fingerprint: [u8; 32]| zcash_voting::prelude::DelegationSigningRequest {
            account_index: 0,
            network: Network::Regtest,
            seed_fingerprint,
            sighash: [9u8; 32],
            alpha: [
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0,
            ],
        };
        assert!(sign_delegation_request(&seed, request([0u8; 32])).is_err());
        let signature = sign_delegation_request(&seed, request(fingerprint)).expect("signs");
        assert_eq!(signature.len(), 64);
        assert!(signature.iter().any(|&byte| byte != 0));
    }
}
