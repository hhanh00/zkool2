//! Voting configuration transport and authentication.

use anyhow::{anyhow, ensure, Result};
use base64::Engine as _;

use crate::net::http;

#[derive(Debug, serde::Deserialize)]
pub struct ServerRound {
    #[serde(
        alias = "id",
        alias = "vote_round_id",
        deserialize_with = "decode_round_id"
    )]
    pub round_id: String,
    pub title: Option<String>,
    pub status: serde_json::Value,
    /// Absent when the overview omits it. Share tracking then polls without
    /// ever classifying a share overdue or stopping at a boundary, rather
    /// than costing the round list an extra per-round request to find out.
    #[serde(default)]
    pub vote_end_time: Option<u64>,
    pub proposals: Vec<ServerProposal>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ServerProposal {
    pub id: u32,
    pub title: Option<String>,
    #[serde(default)]
    pub options: Vec<ServerProposalOption>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ServerProposalOption {
    pub label: Option<String>,
    pub short_title: Option<String>,
    pub title: Option<String>,
}

impl ServerProposalOption {
    pub fn display_label(&self) -> Option<String> {
        [&self.label, &self.short_title, &self.title]
            .into_iter()
            .flatten()
            .find(|label| !label.trim().is_empty())
            .cloned()
    }
}

fn decode_round_id<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let raw = <String as serde::Deserialize>::deserialize(deserializer)?;
    let bytes = if raw.len() == 64 && raw.bytes().all(|b| b.is_ascii_hexdigit()) {
        hex::decode(&raw).map_err(serde::de::Error::custom)?
    } else {
        base64::engine::general_purpose::STANDARD
            .decode(&raw)
            .map_err(serde::de::Error::custom)?
    };
    if bytes.len() != 32 {
        return Err(serde::de::Error::custom("round ID must contain 32 bytes"));
    }
    Ok(hex::encode(bytes))
}

impl ServerRound {
    pub fn display_title(&self) -> String {
        if let Some(title) = self.title.as_ref().filter(|t| !t.trim().is_empty()) {
            return title.clone();
        }
        let titles: Vec<_> = self
            .proposals
            .iter()
            .filter_map(|p| p.title.as_deref().filter(|t| !t.trim().is_empty()))
            .collect();
        if titles.is_empty() {
            format!("Round {}…", &self.round_id[..12])
        } else {
            titles.join(" · ")
        }
    }
}

/// Fetches nonterminal rounds, rotating through equivalent servers on failure.
pub async fn fetch_rounds(
    servers: &[String],
    client: &reqwest::Client,
) -> Result<Vec<ServerRound>> {
    #[derive(serde::Deserialize)]
    struct Envelope {
        #[serde(default)]
        current_rounds: Vec<ServerRound>,
    }
    let urls: Vec<_> = servers
        .iter()
        .map(|server| {
            format!(
                "{}/shielded-vote/v1/rounds/overview",
                server.trim_end_matches('/')
            )
        })
        .collect();
    let response = http::http_get(client, &urls, http::RetryPolicy::default()).await?;
    Ok(response.json::<Envelope>().await?.current_rounds)
}

/// Resolves and authenticates the voting config for a source URL.
pub async fn voting_config_resolve(
    source: &str,
    client: &reqwest::Client,
) -> Result<zcash_voting::config::ResolvedVotingConfig> {
    let source = source.to_string();

    let static_bytes = http::http_get(client, &[&source], http::RetryPolicy::default())
        .await?
        .bytes()
        .await?;
    let resolved_static =
        zcash_voting::config::resolve_static_voting_config(&source, &static_bytes)?;
    let dynamic_bytes = http::http_get(
        client,
        &[&resolved_static.dynamic_config_url],
        http::RetryPolicy::default(),
    )
    .await?
    .bytes()
    .await?;
    let resolved = zcash_voting::config::resolve_dynamic_voting_config(
        resolved_static,
        &dynamic_bytes,
        zcash_voting::config::ResolveVotingConfigOptions::default(),
    )?;

    Ok(resolved)
}

/// Fetches snapshot metadata, binding the authority key to authenticated config.
pub async fn fetch_round_params(
    round_id: &str,
    config: &zcash_voting::config::ResolvedVotingConfig,
    client: &reqwest::Client,
) -> Result<zcash_voting::VotingRoundParams> {
    ensure!(
        round_id.len() == 64 && round_id.bytes().all(|b| b.is_ascii_hexdigit()),
        "invalid round id"
    );
    ensure!(
        config
            .authenticated_rounds
            .iter()
            .any(|round| round.round_id == round_id),
        "round is not authenticated"
    );
    let urls: Vec<_> = config
        .vote_servers
        .iter()
        .map(|server| {
            format!(
                "{}/shielded-vote/v1/round/{round_id}",
                server.url.trim_end_matches('/')
            )
        })
        .collect();
    let response = http::http_get(
        client,
        &urls.iter().map(String::as_str).collect::<Vec<_>>(),
        http::RetryPolicy::default(),
    )
    .await?;
    let body: serde_json::Value = response.json().await?;
    let round = &body["round"];
    let snapshot_height = round["snapshot_height"]
        .as_u64()
        .or_else(|| round["snapshot_height"].as_str()?.parse().ok())
        .ok_or_else(|| anyhow!("round snapshot height is missing or invalid"))?;
    fn root(round: &serde_json::Value, name: &str) -> Result<Vec<u8>> {
        let value = round[name]
            .as_str()
            .ok_or_else(|| anyhow!("round {name} is missing"))?;
        let bytes = if value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit()) {
            hex::decode(value)?
        } else {
            base64::engine::general_purpose::STANDARD.decode(value)?
        };
        Ok(bytes)
    }
    Ok(config.trusted_voting_round_params(
        round_id.to_owned(),
        snapshot_height,
        root(round, "nc_root")?,
        root(round, "nullifier_imt_root")?,
    )?)
}

#[cfg(test)]
mod tests {
    use super::{fetch_rounds, voting_config_resolve};
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    const STATIC_CONFIG_SOURCE: &str = "https://voting.valargroup.dev/pins/stage/046758f8d1f1a74c7ea63461fd77101930c5df5817b74453ed81895c26bf988f/static-voting-config.json?checksum=sha256:046758f8d1f1a74c7ea63461fd77101930c5df5817b74453ed81895c26bf988f";
    const STATIC_CONFIG: &[u8] = include_bytes!("fixtures/static-voting-config.json");

    #[test]
    fn decodes_server_ids_and_uses_proposal_titles() {
        let round: super::ServerRound = serde_json::from_value(serde_json::json!({
            "vote_round_id": "ADq8gEwbOM/IrHul/Hc7tUoY6hvBMfL5WiTJpmdWTAg=",
            "status": 1,
            "proposals": [{"id": 1, "title": "Community funding"}]
        }))
        .unwrap();
        assert_eq!(
            round.round_id,
            "003abc804c1b38cfc8ac7ba5fc773bb54a18ea1bc131f2f95a24c9a667564c08"
        );
        assert_eq!(round.display_title(), "Community funding");
        let hex_round: super::ServerRound = serde_json::from_value(serde_json::json!({
            "round_id": round.round_id.to_uppercase(), "status": 1, "proposals": []
        }))
        .unwrap();
        assert_eq!(hex_round.round_id, round.round_id);
        assert_eq!(hex_round.display_title(), "Round 003abc804c1b…");
        assert!(
            serde_json::from_value::<super::ServerRound>(serde_json::json!({
                "vote_round_id": "YQ==", "status": 1, "proposals": []
            }))
            .is_err()
        );
    }

    #[tokio::test]
    async fn ballot_round_params_use_authenticated_authority() {
        use zcash_voting::config::{
            AuthenticatedRound, ResolvedVotingConfig, ServiceEndpoint, SupportedVersions,
        };
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let round_id = "01".repeat(32);
        let config = ResolvedVotingConfig {
            source_fingerprint: String::new(),
            trusted_key_fingerprint: String::new(),
            dynamic_config_fingerprint: String::new(),
            vote_servers: vec![ServiceEndpoint {
                url,
                label: String::new(),
            }],
            pir_endpoints: vec![],
            pir_layout: Default::default(),
            supported_versions: SupportedVersions {
                pir: vec![],
                vote_protocol: String::new(),
                tally: String::new(),
                vote_server: String::new(),
            },
            authenticated_rounds: vec![AuthenticatedRound {
                round_id: round_id.clone(),
                ea_pk: vec![7; 32],
            }],
            skipped_round_ids: vec![],
            conditions: vec![],
        };
        let requested_id = round_id.clone();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            while !request.ends_with(b"\r\n\r\n") {
                request.push(socket.read_u8().await.unwrap());
            }
            assert!(request.starts_with(
                format!("GET /shielded-vote/v1/round/{requested_id} HTTP/1.1\r\n").as_bytes()
            ));
            let body = serde_json::json!({"round": {
                "snapshot_height": "123", "nc_root": "02".repeat(32),
                "nullifier_imt_root": "AwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwM=",
                "ea_pk": "04".repeat(32)
            }})
            .to_string();
            socket
                .write_all(
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    )
                    .as_bytes(),
                )
                .await
                .unwrap();
        });
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        let params = super::fetch_round_params(&round_id, &config, &client)
            .await
            .unwrap();
        assert_eq!(params.ea_pk, vec![7; 32]);
        assert_eq!(params.snapshot_height, 123);
        assert_eq!(params.nc_root, vec![2; 32]);
        assert_eq!(params.nullifier_imt_root, vec![3; 32]);
        assert!(
            super::fetch_round_params(&"ff".repeat(32), &config, &client)
                .await
                .is_err()
        );
        server.await.unwrap();
    }

    #[tokio::test]
    async fn fetches_round_metadata_with_equivalent_server_fallback() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            while !request.ends_with(b"\r\n\r\n") {
                request.push(socket.read_u8().await.unwrap());
            }
            assert!(request.starts_with(b"GET /shielded-vote/v1/rounds/overview HTTP/1.1\r\n"));
            let body = r#"{"current_rounds":[{"vote_round_id":"ADq8gEwbOM/IrHul/Hc7tUoY6hvBMfL5WiTJpmdWTAg=","title":"Poll","status":1,"proposals":[{"id":1},{"id":2}]}],"completed_round_count":962}"#;
            socket
                .write_all(
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    )
                    .as_bytes(),
                )
                .await
                .unwrap();
        });
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        let rounds = tokio::time::timeout(
            Duration::from_secs(10),
            fetch_rounds(&["invalid-url".into(), url], &client),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(rounds.len(), 1);
        assert_eq!(
            rounds[0].round_id,
            "003abc804c1b38cfc8ac7ba5fc773bb54a18ea1bc131f2f95a24c9a667564c08"
        );
        assert_eq!(rounds[0].title.as_deref(), Some("Poll"));
        assert_eq!(rounds[0].status, 1);
        assert_eq!(rounds[0].proposals[1].id, 2);
        server.await.unwrap();
    }

    #[test]
    fn resolves_pinned_static_config_fixture() {
        let config =
            zcash_voting::config::resolve_static_voting_config(STATIC_CONFIG_SOURCE, STATIC_CONFIG)
                .unwrap();
        assert_eq!(
            config.dynamic_config_url,
            "https://voting.valargroup.dev/stage/dynamic-voting-config.json"
        );

        let mut altered = STATIC_CONFIG.to_vec();
        altered.push(b' ');
        assert!(
            zcash_voting::config::resolve_static_voting_config(STATIC_CONFIG_SOURCE, &altered,)
                .is_err()
        );
    }

    #[tokio::test]
    async fn retries_http_errors_before_resolving_config() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let source = format!("http://{}/static.json", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            for _ in 0..3 {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut request = Vec::new();
                while !request.ends_with(b"\r\n\r\n") {
                    let byte = socket.read_u8().await.unwrap();
                    request.push(byte);
                }
                assert!(request.starts_with(b"GET /static.json HTTP/1.1\r\n"));
                socket
                    .write_all(
                        b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                    )
                    .await
                    .unwrap();
            }
        });
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        let error = tokio::time::timeout(
            Duration::from_secs(10),
            voting_config_resolve(&source, &client),
        )
        .await
        .expect("config fetch timed out")
        .unwrap_err();
        assert!(error.to_string().contains("503"), "{error}");
        tokio::time::timeout(Duration::from_secs(1), server)
            .await
            .expect("expected three GET attempts")
            .unwrap();
    }
}
