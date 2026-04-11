use anyhow::{Result, anyhow};
use blake2b_simd::Params as Blake2bParams;

use bech32::{self, Bech32m, Hrp};

use crate::tiu;

/// Generate seed from passkey PRF output (deterministic)
///
/// Same PRF input always produces the same seed output.
/// Uses BLAKE2b KDF with a fixed personalization string.
pub fn derive_seed_from_passkey_prf(prf_output: [u8; 32]) -> [u8; 32] {
    let mut seed = [0u8; 32];
    let hash = Blake2bParams::new()
        .hash_length(32)
        .personal(b"Zcash__PRF__Seed")
        .to_state()
        .update(&prf_output)
        .finalize();
    seed.copy_from_slice(hash.as_bytes());
    seed
}

/// Encode seed as recovery code (Bech32m, zpass prefix)
///
/// The same seed always produces the same recovery code.
/// Format: zpass1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
pub fn encode_recovery_code(seed: [u8; 32]) -> String {
    let hrp = Hrp::parse_unchecked("zpass");
    bech32::encode::<Bech32m>(hrp, &seed)
        .unwrap()
}

/// Decode recovery code to seed
///
/// Decodes Bech32m zpass recovery code back to 32-byte seed.
/// Returns an error if the code is invalid.
pub fn decode_recovery_code(code: &str) -> Result<[u8; 32]> {
    let (hrp, seed) = bech32::decode(code)
        .map_err(|_| anyhow!("Invalid recovery code"))?;

    if hrp != Hrp::parse_unchecked("zpass") {
        return Err(anyhow!("Invalid recovery code prefix"));
    }

    Ok(tiu!(seed))
}
