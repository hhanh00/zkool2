//! Prints the delegation circuit fingerprint and deterministic proof
//! fingerprints under the APP's patched deps. Compare with the same probes
//! run from the fork workspace (crates.io deps).
fn main() {
    println!(
        "delegation circuit fingerprint: {}",
        zcash_voting::zkp1::delegation_circuit_fingerprint()
    );
    let (len, proof_hex, pi_hex) = zcash_voting::zkp1::delegation_proof_probe();
    println!("delegation proof size: {len} bytes");
    println!("delegation proof sha256[..16]: {proof_hex}");
    println!("public inputs sha256[..8]: {pi_hex}");
}
