use anyhow::Result;
#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;
use zcash_protocol::consensus::Parameters;

use crate::api::coin::Coin;
use crate::openalias;
use crate::pay::Recipient;

/// Resolve an OpenAlias name and return Zcash [`Recipient`]s valid for
/// the wallet's network.
///
/// Performs DNS TXT lookup, parses OA1 records, filters for Zcash
/// addresses, and validates them against the wallet's network type.
#[cfg_attr(feature = "flutter", frb)]
pub async fn resolve_openalias(
    alias: String,
    c: &Coin,
) -> Result<Vec<Recipient>> {
    let net = c.network().network_type();
    openalias::resolve_zcash_for_network(&alias, net).await
}

/// Resolve an OpenAlias name and return ALL cryptocurrency addresses
/// found (not just Zcash) as [`Recipient`]s.
#[cfg_attr(feature = "flutter", frb)]
pub async fn resolve_openalias_all(alias: String) -> Result<Vec<Recipient>> {
    openalias::resolve(&alias).await
}

/// Validate whether a string looks like a valid OpenAlias name format.
/// Returns true/false without performing any DNS lookup.
#[cfg_attr(feature = "flutter", frb(sync))]
pub fn validate_openalias_name(alias: &str) -> bool {
    openalias::validate_alias(alias).is_some()
}

/// Validate that an address string is a syntactically valid Zcash address
/// for the wallet's network (convenience wrapper returning bool).
///
/// See [`try_validate_zcash_address`] for the `Result`-returning variant
/// that provides error details.
#[cfg_attr(feature = "flutter", frb(sync))]
pub fn validate_zcash_address(address: &str, c: &Coin) -> bool {
    let net = c.network().network_type();
    openalias::validate_zcash_address(address, net)
}

/// Try to validate that an address string is a syntactically valid Zcash
/// address for the wallet's network, returning `Ok(())` or an error with
/// details about why validation failed.
#[cfg_attr(feature = "flutter", frb(sync))]
pub fn try_validate_zcash_address(address: String, c: &Coin) -> Result<()> {
    let net = c.network().network_type();
    openalias::try_validate_zcash_address(&address, net)
}

/// Get the raw OpenAlias TXT record strings for diagnostic purposes.
#[cfg_attr(feature = "flutter", frb)]
pub async fn resolve_openalias_raw(alias: String) -> Result<Vec<String>> {
    openalias::resolve_raw(&alias).await
}
