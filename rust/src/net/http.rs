use std::sync::Arc;

use anyhow::{Context, Result};
use arti_client::{TorClient, TorClientConfig};
use httparse::Status;
use rustls::{ClientConfig, RootCertStore, crypto::ring::default_provider, pki_types::ServerName};
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_rustls::TlsConnector;
use webpki_roots::TLS_SERVER_ROOTS;

pub async fn test_tor_get() -> Result<()> {
    let provider = default_provider();

    provider.install_default().unwrap();
    let config = TorClientConfig::default();

    let tor_client = TorClient::create_bootstrapped(config).await?;

    let stream = tor_client.connect(("httpbin.org", 443)).await?;

    let root_cert_store = RootCertStore::from_iter(
        TLS_SERVER_ROOTS.iter().cloned()
    );

    let tls_config = ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth(); // We don't need client certificates for standard web browsing

    let connector = TlsConnector::from(Arc::new(tls_config));

    let server_name: ServerName = "httpbin.org"
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid DNS name"))?;

    let mut tls_stream = connector
        .connect(server_name, stream)
        .await
        .context("TLS handshake failed over Tor stream")?;

    println!("TLS Handshake successful. Sending HTTPS request...");

    tls_stream
        .write_all(b"GET /get HTTP/1.1\r\nHost: httpbin.org\r\nConnection: close\r\n\r\n")
        .await?;

    tls_stream.flush().await?;

    // Read and print the result.
    let mut buf = Vec::new();
    tls_stream.read_to_end(&mut buf).await?;

    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut rep = httparse::Response::new(&mut headers);
    let Status::Complete(offset) = rep.parse(&buf)? else {
        panic!()
    };
    let body = String::from_utf8_lossy(&buf[offset..]);

    let body: Value = serde_json::from_str(&body)?;
    println!("{body}");

    Ok(())
}
