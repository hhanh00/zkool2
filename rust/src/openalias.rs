use std::sync::OnceLock;

use anyhow::{anyhow, Context, Result};
use hickory_resolver::TokioAsyncResolver;
use tracing::info;
use zcash_address::{ConversionError, TryFromAddress, ZcashAddress};
use zcash_protocol::consensus::NetworkType;

use crate::pay::Recipient;

/// Marker type for network validation via `convert_if_network`.
/// All `TryFromAddress` methods use the default (return `Unsupported`),
/// because we only care whether the network check passes.
struct NetCheck;

impl TryFromAddress for NetCheck {
    type Error = ();
}

impl From<openalias::CryptoAddress> for Recipient {
    fn from(addr: openalias::CryptoAddress) -> Self {
        Self {
            address: addr.address,
            user_memo: addr.tx_description,
            ..Default::default()
        }
    }
}

/// Cached async DNS resolver, created once and reused.
fn resolver() -> &'static TokioAsyncResolver {
    static RESOLVER: OnceLock<TokioAsyncResolver> = OnceLock::new();
    RESOLVER.get_or_init(|| {
        TokioAsyncResolver::tokio_from_system_conf()
            .expect("failed to create DNS resolver")
    })
}

/// Perform async DNS TXT lookup and parse OA1 records into [`CryptoAddress`]es.
async fn lookup_oa1_records(alias: &str) -> Result<Vec<openalias::CryptoAddress>> {
    let fqdn = openalias::alias_to_fqdn(alias)
        .ok_or_else(|| anyhow!("Invalid OpenAlias name: {alias}"))?;
    info!("Resolving OpenAlias: {alias} → {fqdn}");

    let response = resolver()
        .txt_lookup(&fqdn)
        .await
        .with_context(|| format!("DNS lookup failed for OpenAlias: {alias}"))?;

    let records: Vec<String> = response
        .iter()
        .flat_map(|txt| txt.iter())
        .filter(|s| s.starts_with(b"oa1:"))
        .map(|s| String::from_utf8_lossy(s).to_string())
        .collect();
    info!("OpenAlias DNS returned {} OA1 record(s) for {alias}: {records:?}", records.len());

    let mut addrs = Vec::new();
    for r in &records {
        match r.trim().parse::<openalias::CryptoAddress>() {
            Ok(addr) => addrs.push(addr),
            Err(e) => info!("OpenAlias failed to parse OA1 record for {alias}: {r} — {e:?}"),
        }
    }
    info!("OpenAlias parsed {} address(es) for {alias}", addrs.len());

    Ok(addrs)
}

/// Resolve an OpenAlias name and return all found cryptocurrency addresses
/// as [`Recipient`]s.
///
/// Performs async DNS TXT lookup and parses OA1 records.
pub async fn resolve(alias: &str) -> Result<Vec<Recipient>> {
    let addrs = lookup_oa1_records(alias).await?;
    let recipients: Vec<Recipient> = addrs.into_iter().map(Into::into).collect();
    info!("OpenAlias {alias} resolved to {} total recipient(s)", recipients.len());
    Ok(recipients)
}

/// Resolve an OpenAlias name and return only Zcash addresses as [`Recipient`]s.
/// Filters by the `cryptocurrency` field being "zcash" (case-insensitive).
pub async fn resolve_zcash(alias: &str) -> Result<Vec<Recipient>> {
    let addrs = lookup_oa1_records(alias).await?;
    let zcash: Vec<_> = addrs
        .into_iter()
        .filter(|a| a.cryptocurrency.eq_ignore_ascii_case("zcash"))
        .map(Into::into)
        .collect();
    if zcash.is_empty() {
        info!("No Zcash address found in OpenAlias result for {alias}");
        Err(anyhow!(
            "No Zcash address found for OpenAlias: {alias}"
        ))
    } else {
        info!("OpenAlias resolved {alias} to {} Zcash recipient(s): {:?}", zcash.len(), zcash);
        Ok(zcash)
    }
}

/// Try to validate that an address string is a syntactically valid Zcash
/// address for the given network type.
///
/// Uses `ZcashAddress::try_from_encoded` for structural validation and
/// `convert_if_network` for the network check. Returns `Ok(())` if the
/// address is valid for the given network, or an error with details.
pub fn try_validate_zcash_address(address: &str, net: NetworkType) -> Result<()> {
    let addr = ZcashAddress::try_from_encoded(address)
        .map_err(|e| anyhow!("Invalid Zcash address: {e}"))?;

    match addr.convert_if_network::<NetCheck>(net) {
        Err(ConversionError::IncorrectNetwork { expected, actual }) => Err(anyhow!(
            "Address {address} is for {actual:?} but we expected {expected:?}"
        )),
        _ => Ok(()),
    }
}

/// Validate that an address string is a syntactically valid Zcash address
/// for the given network type (convenience wrapper returning bool).
pub fn validate_zcash_address(address: &str, net: NetworkType) -> bool {
    try_validate_zcash_address(address, net).is_ok()
}

/// Resolve an OpenAlias name and return validated Zcash [`Recipient`]s for the
/// given network. Filters by cryptocurrency field AND validates the address
/// string against the network type.
pub async fn resolve_zcash_for_network(
    alias: &str,
    net: NetworkType,
) -> Result<Vec<Recipient>> {
    let zcash_addrs = resolve_zcash(alias).await?;
    let total = zcash_addrs.len();
    let valid: Vec<_> = zcash_addrs
        .into_iter()
        .filter(|a| validate_zcash_address(&a.address, net))
        .collect();
    if valid.is_empty() {
        info!(
            "No {net:?} Zcash address found for {alias} \
             ({total} zcash addresses found but none valid for this network)",
        );
        Err(anyhow!(
            "No valid Zcash {net:?} address found for OpenAlias: {alias}"
        ))
    } else {
        info!("OpenAlias {alias} → {valid:?} valid {net:?} recipient(s)");
        Ok(valid)
    }
}

/// Get the raw OpenAlias TXT record strings for an alias (without parsing).
pub async fn resolve_raw(alias: &str) -> Result<Vec<String>> {
    let fqdn = openalias::alias_to_fqdn(alias)
        .ok_or_else(|| anyhow!("Invalid OpenAlias name: {alias}"))?;

    let response = resolver()
        .txt_lookup(&fqdn)
        .await
        .with_context(|| format!("DNS lookup failed for OpenAlias: {alias}"))?;

    Ok(response
        .iter()
        .flat_map(|txt| txt.iter())
        .filter(|s| s.starts_with(b"oa1:"))
        .map(|s| String::from_utf8_lossy(s).to_string())
        .collect())
}

/// Validate an OpenAlias name format.
/// Returns `Some(FQDN)` if valid, `None` if invalid.
/// No DNS lookup is performed.
pub fn validate_alias(alias: &str) -> Option<String> {
    openalias::alias_to_fqdn(alias).map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alias_to_fqdn_valid() {
        assert!(validate_alias("donate@example.org").is_some());
    }

    #[test]
    fn test_alias_to_fqdn_bare_domain() {
        assert!(validate_alias("nabijaczleweli.xyz").is_some());
    }

    #[test]
    fn test_alias_to_fqdn_spaces_invalid() {
        assert!(validate_alias("not an alias").is_none());
    }

    #[test]
    fn test_validate_zcash_address_garbage() {
        assert!(!validate_zcash_address(
            "not-a-zcash-address",
            NetworkType::Main
        ));
        assert!(!validate_zcash_address(
            "not-a-zcash-address",
            NetworkType::Test
        ));
        assert!(!validate_zcash_address(
            "not-a-zcash-address",
            NetworkType::Regtest
        ));
    }
}
