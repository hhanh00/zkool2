#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

/// Offline address validation for a specified blockchain. Ethereum validation
/// checks hex syntax only; this revision does not enforce EIP-55 checksums.
#[cfg_attr(feature = "flutter", frb(sync))]
pub fn validate_blockchain_address(address: &str, blockchain: &str) -> bool {
    let address = address.trim();
    // Bound the upstream Base58 decoder's work for untrusted form input.
    if address.is_empty() || address.len() > 256 {
        return false;
    }
    let chain = blockchain.trim().to_ascii_lowercase();
    let chain = match chain.as_str() {
        "eth" => "ethereum",
        "sol" => "solana",
        "trx" => "tron",
        "btc" => "bitcoin",
        other => other,
    };
    wallet_address_validator_kit::validate_address(address, chain).valid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_solana_decoded_length() {
        assert!(validate_blockchain_address(
            "11111111111111111111111111111111",
            "sol"
        ));
        assert!(!validate_blockchain_address(
            "111111111111111111111111111111111",
            "solana"
        ));
        assert!(!validate_blockchain_address("0OIl", "solana"));
    }

    #[test]
    fn validates_tron_checksum_and_network() {
        let address = "T9yD14Nj9j7xAB4dbGeiX9h8unkKHxuWwb";
        assert!(validate_blockchain_address(address, "tron"));
        assert!(!validate_blockchain_address(
            "T9yD14Nj9j7xAB4dbGeiX9h8unkKHxuWwc",
            "tron"
        ));
        assert!(!validate_blockchain_address(address, "bitcoin"));
    }

    #[test]
    fn ethereum_validation_is_syntax_only() {
        assert!(validate_blockchain_address(
            "0xde709f2102306220921060314715629080e2fb77",
            "eth"
        ));
        // Upstream does not validate mixed-case EIP-55 checksums.
        assert!(validate_blockchain_address(
            "0xDE709f2102306220921060314715629080e2fb77",
            "ethereum"
        ));
        assert!(!validate_blockchain_address("0x1234", "ethereum"));
        assert!(!validate_blockchain_address(
            "0xge709f2102306220921060314715629080e2fb77",
            "ethereum"
        ));
    }

    #[test]
    fn rejects_unknown_chains_empty_and_oversized_input() {
        assert!(!validate_blockchain_address(
            "11111111111111111111111111111111",
            "unknown"
        ));
        assert!(!validate_blockchain_address(" ", "solana"));
        assert!(!validate_blockchain_address(&"1".repeat(257), "solana"));
    }
}
