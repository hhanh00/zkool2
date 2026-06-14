use std::sync::OnceLock;

use anyhow::{anyhow, Context, Result};
use hickory_resolver::TokioAsyncResolver;
use tracing::info;
use zcash_address::{ConversionError, TryFromAddress, ZcashAddress};
use zcash_protocol::consensus::NetworkType;

use crate::pay::Recipient;

// ---------------------------------------------------------------------------
// OA1 record parser (replaces openalias crate)
// ---------------------------------------------------------------------------

/// Minimal parsed OA1 record – only the fields zkool uses.
struct Oa1Record {
    cryptocurrency: String,
    address: String,
    tx_description: Option<String>,
}

impl From<Oa1Record> for Recipient {
    fn from(addr: Oa1Record) -> Self {
        Self {
            address: addr.address,
            user_memo: addr.tx_description,
            ..Default::default()
        }
    }
}

/// Split an OA1 record body on `;` while respecting double-quoted sections
/// (semicolons inside quotes are not separators).
fn split_oa1_pairs(body: &str) -> Vec<&str> {
    let mut pairs = Vec::new();
    let mut start = 0;
    let mut in_quotes = false;
    let bytes = body.as_bytes();

    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'"' => in_quotes = !in_quotes,
            b';' if !in_quotes => {
                let pair = body[start..i].trim();
                if !pair.is_empty() {
                    pairs.push(pair);
                }
                start = i + 1;
            }
            _ => {}
        }
    }
    let last = body[start..].trim();
    if !last.is_empty() {
        pairs.push(last);
    }
    pairs
}

/// Parse a single `key=value` pair from an OA1 record.
///
/// Handles double-quoted values (quotes stripped) and backslash-escaped
/// spaces in unquoted values.
fn parse_oa1_pair(pair: &str) -> Option<(String, String)> {
    let (key, val) = pair.split_once('=')?;
    let key = key.trim().to_string();

    let val = val.trim();
    let val = if val.starts_with('"') && val.ends_with('"') && val.len() >= 2 {
        val[1..val.len() - 1].to_string()
    } else {
        val.replace("\\ ", " ")
    };

    Some((key, val))
}

/// Parse an OA1 TXT record string into an [`Oa1Record`].
///
/// Format: `oa1:<crypto> key1=val1; key2="val2"; ...`
fn parse_oa1(input: &str) -> Option<Oa1Record> {
    let input = input.trim();

    // Strip "oa1:" prefix
    let after_prefix = input.strip_prefix("oa1:")?;

    // Extract crypto name (lowercase letters before the first space)
    let (crypto, rest) = after_prefix.split_once(' ')?;
    let crypto = crypto.trim();
    if crypto.is_empty() || !crypto.chars().all(|c| c.is_ascii_lowercase()) {
        return None;
    }

    let mut address: Option<String> = None;
    let mut tx_description: Option<String> = None;

    for raw_pair in split_oa1_pairs(rest) {
        let (key, val) = parse_oa1_pair(raw_pair)?;
        match key.as_str() {
            "recipient_address" => address = Some(val),
            "tx_description" => tx_description = Some(val),
            // All other keys (recipient_name, tx_amount, tx_payment_id,
            // address_signature, checksum, and custom keys) are ignored as
            // they are not used by zkool.
            _ => {}
        }
    }

    Some(Oa1Record {
        cryptocurrency: crypto.to_string(),
        address: address?,
        tx_description,
    })
}

/// Convert an OpenAlias to an FQDN per the
/// [OpenAlias spec](https://openalias.org#implement):
///
/// 1. Replace `@` with `.` (email-style addressing).
/// 2. Require at least one `.` in the result.
/// 3. Append a trailing dot if absent (FQDN).
fn alias_to_fqdn(alias: &str) -> Option<String> {
    let mut alias = alias.replace('@', ".");
    if alias.contains('.') {
        if !alias.ends_with('.') {
            alias.push('.');
        }
        Some(alias)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Marker type for network validation via `convert_if_network`.
// All `TryFromAddress` methods use the default (return `Unsupported`),
// because we only care whether the network check passes.
// ---------------------------------------------------------------------------

struct NetCheck;

impl TryFromAddress for NetCheck {
    type Error = ();
}

/// Cached async DNS resolver, created once and reused.
fn resolver() -> &'static TokioAsyncResolver {
    static RESOLVER: OnceLock<TokioAsyncResolver> = OnceLock::new();
    RESOLVER.get_or_init(|| {
        // Try system config first (reads /etc/resolv.conf on Unix).
        // On Android the resolv.conf file may not exist, so fall back
        // to a built-in resolver config using Google's public DNS.
        TokioAsyncResolver::tokio_from_system_conf().unwrap_or_else(|e| {
            info!(
                "Failed to create DNS resolver from system config ({e}); \
                 falling back to Google public DNS"
            );
            TokioAsyncResolver::tokio(
                hickory_resolver::config::ResolverConfig::google(),
                hickory_resolver::config::ResolverOpts::default(),
            )
        })
    })
}

/// Perform async DNS TXT lookup and parse OA1 records into [`Oa1Record`]s.
async fn lookup_oa1_records(alias: &str) -> Result<Vec<Oa1Record>> {
    let fqdn =
        alias_to_fqdn(alias).ok_or_else(|| anyhow!("Invalid OpenAlias name: {alias}"))?;
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
    info!(
        "OpenAlias DNS returned {} OA1 record(s) for {alias}: {records:?}",
        records.len()
    );

    let mut addrs = Vec::new();
    for r in &records {
        match parse_oa1(r) {
            Some(addr) => addrs.push(addr),
            None => info!(
                "OpenAlias failed to parse OA1 record for {alias}: {r}"
            ),
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
    info!(
        "OpenAlias {alias} resolved to {} total recipient(s)",
        recipients.len()
    );
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
        Err(anyhow!("No Zcash address found for OpenAlias: {alias}"))
    } else {
        info!(
            "OpenAlias resolved {alias} to {} Zcash recipient(s): {:?}",
            zcash.len(),
            zcash
        );
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
    let addr =
        ZcashAddress::try_from_encoded(address).map_err(|e| anyhow!("Invalid Zcash address: {e}"))?;

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
    let fqdn =
        alias_to_fqdn(alias).ok_or_else(|| anyhow!("Invalid OpenAlias name: {alias}"))?;

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
    alias_to_fqdn(alias).map(|s| s.to_string())
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
        assert!(!validate_zcash_address("not-a-zcash-address", NetworkType::Main));
        assert!(!validate_zcash_address("not-a-zcash-address", NetworkType::Test));
        assert!(!validate_zcash_address(
            "not-a-zcash-address",
            NetworkType::Regtest
        ));
    }

    #[test]
    fn test_parse_oa1_simple() {
        let record = "oa1:btc recipient_address=1MoSyGZp3SKpoiXPXfZDFK7cDUFCVtEDeS;";
        let parsed = parse_oa1(record).unwrap();
        assert_eq!(parsed.cryptocurrency, "btc");
        assert_eq!(parsed.address, "1MoSyGZp3SKpoiXPXfZDFK7cDUFCVtEDeS");
        assert_eq!(parsed.tx_description, None);
    }

    #[test]
    fn test_parse_oa1_with_quoted_semicolon() {
        let record =
            "oa1:btc recipient_address=1addr; recipient_name=\"nabijaczleweli; FOSS\";";
        let parsed = parse_oa1(record).unwrap();
        assert_eq!(parsed.cryptocurrency, "btc");
        assert_eq!(parsed.address, "1addr");
    }

    #[test]
    fn test_parse_oa1_escaped_space() {
        let record = "oa1:btc recipient_address=1addr; tx_description=hello\\ world;";
        let parsed = parse_oa1(record).unwrap();
        assert_eq!(parsed.tx_description, Some("hello world".to_string()));
    }

    #[test]
    fn test_parse_oa1_full() {
        let record = "oa1:btc recipient_address=1MoSyGZp3SKpoiXPXfZDFK7cDUFCVtEDeS; recipient_name=\"nabijaczleweli; FOSS development\"; tx_description=Donation for nabijaczleweli:\\ ; tx_amount=0.1;checksum=D851342C; kaschism=yass;";
        let parsed = parse_oa1(record).unwrap();
        assert_eq!(parsed.cryptocurrency, "btc");
        assert_eq!(parsed.address, "1MoSyGZp3SKpoiXPXfZDFK7cDUFCVtEDeS");
        assert_eq!(parsed.tx_description, Some("Donation for nabijaczleweli: ".to_string()));
    }

    #[test]
    fn test_parse_oa1_not_oa1() {
        assert!(parse_oa1("v=spf1 include:_spf.example.com ~all").is_none());
        assert!(parse_oa1("not an oa1 record").is_none());
    }
}
