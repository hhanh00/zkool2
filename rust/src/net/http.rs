use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use arti_client::{TorClient, TorClientConfig};
use httparse::Status;
use rustls::{crypto::ring::default_provider, pki_types::ServerName, ClientConfig, RootCertStore};
use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_rustls::TlsConnector;
use webpki_roots::TLS_SERVER_ROOTS;

use crate::{
    api::coin::get_tor_client,
    IntoAnyhow,
};

pub struct TorHttpClient {
    host: String,
    port: u16,
    uri: String,
    tls_config: Arc<ClientConfig>,
}

impl TorHttpClient {
    pub fn new(host: &str, port: u16, uri: &str) -> Self {
        let root_cert_store = RootCertStore::from_iter(TLS_SERVER_ROOTS.iter().cloned());

        let tls_config = ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth(); // We don't need client certificates for standard web browsing

        Self {
            host: host.to_string(),
            port,
            uri: uri.to_string(),
            tls_config: Arc::new(tls_config),
        }
    }

    pub async fn post_tor(&self, method: &str, params: Value) -> Result<Value> {
        let tor_client = get_tor_client().await.lock().await;

        let connector = TlsConnector::from(self.tls_config.clone());

        let host = self.host.clone();
        let server_name: ServerName = host.clone().try_into().anyhow()?;

        let stream = tor_client.connect((host, self.port)).await?;

        let mut tls_stream = connector
            .connect(server_name, stream)
            .await
            .context("TLS handshake failed over Tor stream")?;

        let id = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let params = serde_json::to_string(&params)?;
        let request_json = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method.to_string(),
            "params": params,
        }).to_string();

        tls_stream
            .write_all(format!("POST /{} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{request_json}", self.uri, self.host).as_bytes())
            .await?;

        tls_stream.flush().await?;

        let mut buf = Vec::new();
        tls_stream.read_to_end(&mut buf).await?;

        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut rep = httparse::Response::new(&mut headers);
        let Status::Complete(offset) = rep.parse(&buf)? else {
            anyhow::bail!("Invalid HTTP response")
        };
        let body = String::from_utf8_lossy(&buf[offset..]);

        let body: Value = serde_json::from_str(&body)?;
        if let Some(error) = body.pointer("/error") {
            anyhow::bail!(
                "JSON RPC error: {}",
                error.pointer("/message").unwrap().as_str().unwrap()
            )
        }
        let result = body.pointer("/result").unwrap();

        Ok(result.clone())
    }
}

pub async fn test_tor_get() -> Result<()> {
    let provider = default_provider();

    provider.install_default().unwrap();
    let config = TorClientConfig::default();

    let tor_client = TorClient::create_bootstrapped(config).await?;

    let stream = tor_client.connect(("httpbin.org", 443)).await?;

    let root_cert_store = RootCertStore::from_iter(TLS_SERVER_ROOTS.iter().cloned());

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
