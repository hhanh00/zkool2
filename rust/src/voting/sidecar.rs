//! Async adapter over the voting crate's blocking sidecar API.
//!
//! `zcash_voting` is synchronous and owns its own rusqlite sidecar database
//! next to the wallet file. Every call therefore runs on a blocking thread:
//! the crate's HTTP transport drives its own Tokio runtime internally, and
//! `Runtime::block_on` panics inside an async task but is fine on a
//! `spawn_blocking` thread.
//!
//! zkool never lends the crate a `WalletDb`. It supplies its own notes,
//! keys and Merkle witnesses, so only the caller-supplied half of the crate's
//! API is used here.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, LazyLock, Mutex,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{ensure, Result};
use futures::StreamExt;
use zcash_voting::delegate::{ensure_round_context, DelegationLwdInputs};
use zcash_voting::prelude::{
    bundle_notes_for_index_for_round, BundlePolicy, Decision, DelegationKeys, DraftVote, Network,
    NoteInfo, PreparedDelegationBundle, VotingDb, VotingError, WitnessData,
};
use zcash_voting::{
    ChainSubmissionControl, HelperClient, HelperFuture, HelperHealth, HelperResponse,
    HelperTransport, HelperTransportError, NoopShareTrackingReporter, ShareTrackingDriver,
    ShareTrackingHostContext, ShareTrackingHostSourceBridge, MAX_HELPER_RESPONSE_BYTES,
};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct ShareTrackerKey {
    wallet_id: String,
    round_id: String,
}

#[derive(Clone)]
struct ActiveShareTracker {
    generation: u64,
    control: ChainSubmissionControl,
}

/// Live retry workers are process-local: their durable work queue is in the
/// voting sidecar, so a process restart simply discovers it again. The map is
/// keyed by both wallet and round so a wallet switch cannot affect another
/// wallet's retry worker.
static ACTIVE_SHARE_TRACKERS: LazyLock<Mutex<HashMap<ShareTrackerKey, ActiveShareTracker>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static NEXT_SHARE_TRACKER_GENERATION: AtomicU64 = AtomicU64::new(1);

/// Routes voting-helper HTTP requests through zkool's selected transport.
///
/// Direct connections and externally configured proxies use the wallet's
/// standard reqwest setup. Tor uses the wallet's Arti HTTP adapter; Nym
/// deliberately fails closed until it has a native HTTP adapter.
#[derive(Clone, Debug)]
pub struct ZkoolHelperTransport {
    transport: u8,
    proxy: String,
}

impl ZkoolHelperTransport {
    pub fn new(transport: u8, proxy: impl Into<String>) -> Self {
        Self {
            transport,
            proxy: proxy.into(),
        }
    }

    fn direct_or_proxy_client(
        &self,
        timeout: Duration,
    ) -> Result<reqwest::Client, HelperTransportError> {
        match self.transport {
            0 => crate::net::http::client("", timeout).map_err(|_| {
                HelperTransportError::Transport("failed to create helper route".into())
            }),
            3 => crate::net::http::client(&self.proxy, timeout).map_err(|_| {
                HelperTransportError::Transport("failed to create helper proxy route".into())
            }),
            2 => Err(HelperTransportError::Transport(
                "Nym helper transport is unavailable".into(),
            )),
            _ => Err(HelperTransportError::Transport(
                "unsupported helper transport".into(),
            )),
        }
    }

    async fn send(
        &self,
        request: reqwest::RequestBuilder,
        timeout: Duration,
    ) -> Result<HelperResponse, HelperTransportError> {
        let response = match tokio::time::timeout(timeout, request.send()).await {
            Err(_) => return Err(HelperTransportError::Timeout),
            Ok(Err(error)) if error.is_timeout() => return Err(HelperTransportError::Timeout),
            Ok(Err(error)) if error.is_connect() || error.is_builder() => {
                return Err(HelperTransportError::Transport(
                    "helper connection failed".into(),
                ));
            }
            Ok(Err(_)) => {
                return Err(HelperTransportError::Ambiguous(
                    "helper request ended before response headers".into(),
                ));
            }
            Ok(Ok(response)) => response,
        };
        let status = response.status().as_u16();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        if response
            .content_length()
            .is_some_and(|length| length as usize > MAX_HELPER_RESPONSE_BYTES)
        {
            return Err(HelperTransportError::Response(
                "helper response exceeds the protocol body limit".into(),
            ));
        }
        let mut body = Vec::new();
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|_| {
                HelperTransportError::Response("helper response body read failed".into())
            })?;
            if body.len().saturating_add(chunk.len()) > MAX_HELPER_RESPONSE_BYTES {
                return Err(HelperTransportError::Response(
                    "helper response exceeds the protocol body limit".into(),
                ));
            }
            body.extend_from_slice(&chunk);
        }
        Ok(HelperResponse::new(status, body, content_type))
    }

    async fn tor_send(
        &self,
        method: hyper::Method,
        url: &str,
        body: Vec<u8>,
        headers: &[(String, String)],
        timeout: Duration,
    ) -> Result<HelperResponse, HelperTransportError> {
        match crate::net::http::tor_request(
            method,
            url,
            body,
            headers,
            timeout,
            MAX_HELPER_RESPONSE_BYTES,
        )
        .await
        {
            Ok(response) => Ok(HelperResponse::new(
                response.status,
                response.body,
                response.content_type,
            )),
            Err(error) if error.to_string().contains("timed out") => {
                Err(HelperTransportError::Timeout)
            }
            // A failure reported by Hyper after route construction may have
            // reached the helper. Treat it as ambiguous rather than risking a
            // duplicate share submission on retry.
            Err(error) => Err(HelperTransportError::Ambiguous(error.to_string())),
        }
    }

    async fn request(
        &self,
        method: hyper::Method,
        url: &str,
        body: Vec<u8>,
        headers: &[(String, String)],
        timeout: Duration,
    ) -> Result<HelperResponse, HelperTransportError> {
        if self.transport == 1 {
            return self.tor_send(method, url, body, headers, timeout).await;
        }

        let client = self.direct_or_proxy_client(timeout)?;
        let mut request = client.request(method, url).body(body);
        for (name, value) in headers {
            request = request.header(name, value);
        }
        self.send(request, timeout).await
    }
}

impl HelperTransport for ZkoolHelperTransport {
    fn get<'a>(&'a self, url: &'a str, timeout: Duration) -> HelperFuture<'a> {
        Box::pin(async move {
            self.request(hyper::Method::GET, url, Vec::new(), &[], timeout)
                .await
        })
    }

    fn post_json<'a>(&'a self, url: &'a str, body: Vec<u8>, timeout: Duration) -> HelperFuture<'a> {
        self.post_json_with_headers(url, body, timeout, &[])
    }

    fn post_json_with_headers<'a>(
        &'a self,
        url: &'a str,
        body: Vec<u8>,
        timeout: Duration,
        headers: &'a [(String, String)],
    ) -> HelperFuture<'a> {
        Box::pin(async move {
            let mut request_headers =
                vec![("content-type".to_string(), "application/json".to_string())];
            request_headers.extend(headers.iter().cloned());
            self.request(hyper::Method::POST, url, body, &request_headers, timeout)
                .await
        })
    }
}

#[cfg(test)]
mod helper_transport_tests {
    use super::{HelperTransportError, ZkoolHelperTransport};
    use std::time::Duration;

    #[test]
    fn nym_fails_closed_without_a_native_http_adapter() {
        assert!(matches!(
            ZkoolHelperTransport::new(2, "").direct_or_proxy_client(Duration::from_secs(1)),
            Err(HelperTransportError::Transport(_)),
        ));
    }
}

/// One wallet's handle on the voting sidecar database.
///
/// `VotingDb` is `Send + Sync` (its connection sits behind a mutex), so the
/// handle is cheap to clone and safe to move onto blocking threads. The crate
/// keeps one connection per sidecar path, so cloning this does not open a
/// second connection.
#[derive(Clone)]
pub struct VotingSidecar {
    db: Arc<VotingDb>,
}

/// Names the voting sidecar for a wallet database without opening anything.
///
/// The suffix is the voting crate's to choose, so callers that need to move or
/// remove the file alongside its wallet ask here rather than spelling it out.
pub fn voting_db_path(wallet_db_path: &Path) -> PathBuf {
    VotingDb::wallet_sidecar_path(wallet_db_path)
}

/// Creates the voting sidecar beside `wallet_db_path` and applies its schema.
///
/// The sidecar is `wallet_db_path` with a `.voting` suffix; call
/// `VotingDb::wallet_sidecar_path` to name it without opening it. `open_path`
/// creates the file if it is missing and runs the crate's migrations either
/// way, so this is the whole of voting database initialization -- there is no
/// separate schema step. No wallet id is involved: the handle opened here is
/// unscoped and dropped immediately, since only wallet-scoped row access needs
/// one. Blocking, like every other call into the crate.
pub async fn create_voting_db(wallet_db_path: PathBuf) -> Result<()> {
    let path = VotingDb::wallet_sidecar_path(&wallet_db_path);
    tokio::task::spawn_blocking(move || VotingDb::open_path(&path)).await??;
    Ok(())
}

impl VotingSidecar {
    #[cfg(test)]
    pub(super) fn from_db_for_test(db: VotingDb) -> Self {
        Self { db: Arc::new(db) }
    }

    /// Opens the sidecar beside `wallet_db_path` and binds it to `wallet_id`.
    ///
    /// The crate derives the sidecar filename from the wallet path and runs
    /// migrations on first open, both of which touch the filesystem, so this
    /// runs off the async executor like every other call here.
    pub async fn open(wallet_db_path: PathBuf, wallet_id: String) -> Result<Self> {
        let db = tokio::task::spawn_blocking(move || {
            VotingDb::open_wallet_sidecar(&wallet_db_path, &wallet_id)
        })
        .await??;
        Ok(Self { db })
    }

    /// Runs one blocking voting-crate call on a blocking thread.
    ///
    /// This is the only way this module touches the crate: the closure takes
    /// the database by reference, so callers cannot accidentally hold the
    /// handle across an await point while the connection mutex is locked.
    pub async fn run<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&VotingDb) -> Result<T, VotingError> + Send + 'static,
        T: Send + 'static,
    {
        let db = Arc::clone(&self.db);
        Ok(tokio::task::spawn_blocking(move || f(&db)).await??)
    }

    /// Stores Merkle inclusion witnesses zkool computed from its own note
    /// commitment tree.
    ///
    /// This replaces the crate's `ensure_witnesses`/`note_witnesses` helpers,
    /// which read the wallet through `zcash_client_sqlite`. The crate still
    /// validates each witness against the stored round params, so a witness
    /// built against the wrong anchor is rejected here rather than at proof
    /// time.
    pub async fn store_witnesses(
        &self,
        round_id: String,
        bundle_index: u32,
        witnesses: Vec<WitnessData>,
    ) -> Result<()> {
        self.run(move |db| db.store_witnesses(&round_id, bundle_index, &witnesses))
            .await
    }

    /// Atomically persists a completed ballot before execution begins.
    ///
    /// The UI represents a skipped proposal using the DraftVote convention
    /// `choice == num_options`. The sidecar owns the translation to the
    /// voting crate's durable `Decision`, so a caller cannot accidentally
    /// write a skipped proposal as a cast vote. The crate validates every
    /// proposal id and option count, rejects duplicate proposals, and applies
    /// the entire ballot in one transaction.
    pub async fn save_ballot(
        &self,
        round_id: String,
        network: Network,
        drafts: Vec<DraftVote>,
    ) -> Result<()> {
        let intents = drafts
            .into_iter()
            .map(|draft| {
                let decision = if draft.choice == draft.num_options {
                    Decision::Skipped
                } else {
                    Decision::Choice(draft.choice)
                };
                (draft.proposal_id, decision, draft.num_options)
            })
            .collect::<Vec<_>>();
        self.run(move |db| db.set_ballot_intents(&round_id, network, &intents))
            .await
    }

    /// Lists this wallet's rounds that still have helper shares awaiting
    /// confirmation. This is the durable restart point for share tracking:
    /// callers should enumerate it at startup and after a foreground round
    /// run, rather than retaining timers or in-memory job state.
    pub async fn pending_share_rounds(
        &self,
    ) -> Result<Vec<zcash_voting::share::PendingShareRoundForAccount>> {
        self.run(move |db| {
            let wallet_id = db.wallet_id().to_owned();
            zcash_voting::share::pending_rounds_for_accounts(db, &[&wallet_id])
        })
        .await
    }

    /// Starts durable helper-share tracking for one round.
    ///
    /// Starting an already-live round is a no-op. A cancelled predecessor is
    /// replaced, while its SDK admission guard ensures the replacement waits
    /// for the departing run rather than issuing duplicate helper requests.
    pub fn start_share_tracking(
        &self,
        round_id: String,
        helper_urls: Vec<String>,
        vote_end_time_seconds: Option<u64>,
        transport: u8,
        proxy: String,
    ) -> Result<bool> {
        ensure!(
            !helper_urls.is_empty(),
            "share tracking requires at least one helper URL"
        );
        let key = ShareTrackerKey {
            wallet_id: self.db.wallet_id().to_owned(),
            round_id,
        };
        let generation = NEXT_SHARE_TRACKER_GENERATION.fetch_add(1, Ordering::Relaxed);
        let control = ChainSubmissionControl::new(generation);
        {
            let mut active = ACTIVE_SHARE_TRACKERS
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            if active
                .get(&key)
                .is_some_and(|tracker| !tracker.control.is_cancelled())
            {
                return Ok(false);
            }
            active.insert(
                key.clone(),
                ActiveShareTracker {
                    generation,
                    control: control.clone(),
                },
            );
        }

        let db = Arc::clone(&self.db);
        tokio::spawn(async move {
            let client = HelperClient::new(
                Arc::new(ZkoolHelperTransport::new(transport, proxy)),
                HelperHealth::default(),
            );
            let host = ShareTrackingHostSourceBridge::new(move || ShareTrackingHostContext {
                configured_helper_urls: helper_urls.clone(),
                now_seconds: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                vote_end_time_seconds,
            });
            ShareTrackingDriver::new(&db, &client, &key.round_id)
                .run(&host, &control, &NoopShareTrackingReporter::default())
                .await;

            let mut active = ACTIVE_SHARE_TRACKERS
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            if active
                .get(&key)
                .is_some_and(|tracker| tracker.generation == generation)
            {
                active.remove(&key);
            }
        });
        Ok(true)
    }

    /// Cancels one round's tracking run. The SDK wakes any retry wait rather
    /// than leaving the task alive until its next scheduled pass.
    pub fn cancel_share_tracking(&self, round_id: &str) -> bool {
        let key = ShareTrackerKey {
            wallet_id: self.db.wallet_id().to_owned(),
            round_id: round_id.to_owned(),
        };
        let active = ACTIVE_SHARE_TRACKERS
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let Some(tracker) = active.get(&key) else {
            return false;
        };
        tracker.control.cancel();
        true
    }

    /// Cancels all tracking runs for this wallet, leaving other open wallets
    /// untouched. Used by app background, lock, wallet switch, and close.
    pub fn cancel_all_share_tracking(&self) -> usize {
        let wallet_id = self.db.wallet_id();
        let active = ACTIVE_SHARE_TRACKERS
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let mut cancelled = 0;
        for (key, tracker) in active.iter() {
            if key.wallet_id == wallet_id && !tracker.control.is_cancelled() {
                tracker.control.cancel();
                cancelled += 1;
            }
        }
        cancelled
    }

    /// Prepares one delegation bundle from caller-supplied notes and witnesses.
    ///
    /// Mirrors the crate's own `prepare_delegation_bundle` with the two
    /// wallet-database steps replaced: the scanned height and note selection
    /// come from zkool, and the witnesses are stored directly instead of being
    /// regenerated from a `WalletDb`.
    #[allow(clippy::too_many_arguments)]
    pub async fn prepare_delegation_bundle(
        &self,
        lwd: DelegationLwdInputs,
        session_json: Option<String>,
        round_note_infos: Vec<NoteInfo>,
        delegation_keys: DelegationKeys,
        witnesses: Vec<WitnessData>,
        bundle_index: u32,
        bundle_policy: BundlePolicy,
    ) -> Result<PreparedDelegationBundle> {
        self.run(move |db| {
            let DelegationLwdInputs {
                round_params,
                resolved_round_name,
                anchor_tree_state_bytes,
                branch_id_provider,
                network,
            } = lwd;

            ensure_round_context(
                db,
                network,
                &round_params,
                &resolved_round_name,
                session_json.as_deref(),
            )?;
            // `None`, not `session_json`: the context call above already
            // applied it, and the crate's own prepare path passes `None` here.
            db.ensure_round(network, &round_params, None)?;

            let round_id = round_params.vote_round_id.clone();
            let layout = db.ensure_bundles_with_skipped_suffix_with_policy(
                &round_id,
                &round_note_infos,
                bundle_policy,
            )?;
            let bundle_note_infos = bundle_notes_for_index_for_round(
                &round_note_infos,
                &layout,
                bundle_index,
                db,
                &round_id,
            )?;

            // Witnesses must land before any proof step reads them; the crate
            // validates them against the round params stored above.
            db.store_witnesses(&round_id, bundle_index, &witnesses)?;

            Ok(PreparedDelegationBundle {
                round_id,
                round_params,
                bundle_index,
                layout,
                bundle_note_infos,
                delegation_keys,
                branch_id_provider,
                anchor_tree_state_bytes,
                network,
                round_name: resolved_round_name,
            })
        })
        .await
    }
}
