//! Voting configuration transport and authentication.

use anyhow::Result;

use crate::net::http;

/// Resolves and authenticates the voting config for a source URL.
pub async fn voting_config_resolve(
    source: &str,
    client: &reqwest::Client,
) -> Result<zcash_voting::config::ResolvedVotingConfig> {
    let source = source.to_string();

    let static_bytes = http::http_get(client, &source, http::RetryPolicy::default())
        .await?
        .bytes()
        .await?;
    let resolved_static =
        zcash_voting::config::resolve_static_voting_config(&source, &static_bytes)?;
    let dynamic_bytes = http::http_get(
        client,
        &resolved_static.dynamic_config_url,
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
    use super::voting_config_resolve;
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    const STATIC_CONFIG_SOURCE: &str = "https://voting.valargroup.dev/pins/stage/046758f8d1f1a74c7ea63461fd77101930c5df5817b74453ed81895c26bf988f/static-voting-config.json?checksum=sha256:046758f8d1f1a74c7ea63461fd77101930c5df5817b74453ed81895c26bf988f";
    const STATIC_CONFIG: &[u8] = include_bytes!("fixtures/static-voting-config.json");

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
