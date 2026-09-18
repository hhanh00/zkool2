//! Voting configuration transport and authentication.

use anyhow::Result;

use crate::net::http;

#[derive(Debug, serde::Deserialize)]
pub struct ServerRound {
    #[serde(alias = "id", alias = "vote_round_id")]
    pub round_id: String,
    pub title: Option<String>,
    pub status: serde_json::Value,
    pub proposals: Vec<ServerProposal>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ServerProposal {
    pub id: u32,
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

#[cfg(test)]
mod tests {
    use super::{fetch_rounds, voting_config_resolve};
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    const STATIC_CONFIG_SOURCE: &str = "https://voting.valargroup.dev/pins/stage/046758f8d1f1a74c7ea63461fd77101930c5df5817b74453ed81895c26bf988f/static-voting-config.json?checksum=sha256:046758f8d1f1a74c7ea63461fd77101930c5df5817b74453ed81895c26bf988f";
    const STATIC_CONFIG: &[u8] = include_bytes!("fixtures/static-voting-config.json");

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
            let body = r#"{"current_rounds":[{"round_id":"r","title":"Poll","status":1,"proposals":[{"id":1},{"id":2}]}],"completed_round_count":962}"#;
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
        assert_eq!(rounds[0].round_id, "r");
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
