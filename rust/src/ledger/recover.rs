use anyhow::Result;
use bip39::Mnemonic;
use sapling_crypto::zip32::ExtendedSpendingKey;
use tracing::debug;
use zcash_protocol::{
    consensus::{MainNetwork, NetworkConstants},
};

use hmac::{Hmac, Mac};
use sha2::{Sha256, Sha512};
use zip32::ChildIndex;

type HmacSha256 = Hmac<Sha256>;
type HmacSha512 = Hmac<Sha512>;

/// Extended private key: 64-byte (kL||kR) + 32-byte chain code
#[derive(Debug, Clone)]
pub struct ExtendedPrivateKey {
    pub key: [u8; 64],
    pub chain_code: [u8; 32],
}

/// Replicates the Ledger logic in the C snippet:
/// - chain_code = HMAC-SHA256(sk, 0x01 || seed)
/// - I = HMAC-SHA512(sk, seed)
/// - while (I[31] & 0x20) { I = HMAC-SHA512(sk, I) }
/// - private_scalar = clamp(I[0..32])
pub fn ledger_derivation_from_seed(seed: &[u8]) -> Result<ExtendedPrivateKey> {
    debug!("MS {}", hex::encode(seed));
    // 1) chain code = HMAC-SHA256(sk, 0x01 || seed)
    let sk = b"ed25519 seed";
    let mut cc_mac =
        HmacSha256::new_from_slice(sk).map_err(|e| anyhow::anyhow!("HMAC key error: {e}"))?;
    cc_mac.update(&[0x01]);
    cc_mac.update(seed);
    let cc_bytes = cc_mac.finalize().into_bytes(); // 32 bytes
    let mut chain_code = [0u8; 32];
    chain_code.copy_from_slice(&cc_bytes);
    debug!("cc {}", hex::encode(chain_code));

    // 2) I = HMAC-SHA512(sk, seed)
    let mut i_mac =
        HmacSha512::new_from_slice(sk).map_err(|e| anyhow::anyhow!("HMAC key error: {e}"))?;
    i_mac.update(seed);
    let i_bytes = i_mac.finalize().into_bytes(); // 64 bytes
    let mut raw_i = [0u8; 64];
    raw_i.copy_from_slice(&i_bytes);
    debug!("I {}", hex::encode(raw_i));

    // 3) while (I[31] & 0x20) { I = HMAC-SHA512(sk, I) }
    // Note: i_bytes[31] corresponds to raw_i[31].
    while (raw_i[31] & 0x20) != 0 {
        debug!("RETRY");
        let mut mac =
            HmacSha512::new_from_slice(sk).map_err(|e| anyhow::anyhow!("HMAC key error: {e}"))?;
        mac.update(&raw_i);
        let new_i = mac.finalize().into_bytes();
        raw_i.copy_from_slice(&new_i);
    }

    // 4) clamp the first 32 bytes (Ed25519 clamping)
    let mut key = [0u8; 64];
    key.copy_from_slice(&raw_i);
    key[0] &= 0xF8;
    key[31] = (key[31] & 0x7F) | 0x40;

    debug!("key {}", hex::encode(key));
    debug!("cc {}", hex::encode(chain_code));

    Ok(ExtendedPrivateKey { key, chain_code })
}

/// Hardened-only Ledger-style BIP32 Ed25519 derivation
pub fn derive_hardened_ed25519(
    parent: &ExtendedPrivateKey,
    path: &[u32], // all indices >= 0x80000000
) -> Result<ExtendedPrivateKey> {
    let mut key = parent.key;
    let mut chain_code = parent.chain_code;

    let mut tmp = [0u8; 1 + 64 + 4]; // prefix + parent key + index
    let mut z = [0u8; 64]; // HMAC output

    for &index in path {
        assert!(index >= 0x80000000, "Hardened-only derivation");

        // HMAC input: 0x00 || parent_key || index (little-endian)
        tmp[0] = 0x00;
        tmp[1..65].copy_from_slice(&key);
        tmp[65] = (index & 0xFF) as u8;
        tmp[66] = ((index >> 8) & 0xFF) as u8;
        tmp[67] = ((index >> 16) & 0xFF) as u8;
        tmp[68] = ((index >> 24) & 0xFF) as u8;

        // Z = HMAC-SHA512(chain_code, tmp)
        let mut mac = HmacSha512::new_from_slice(&chain_code)?;
        mac.update(&tmp[..69]);
        z.copy_from_slice(&mac.finalize().into_bytes());

        // Split Z
        let zl = &z[..28];
        let zr = &z[32..];

        debug!("ZL {}", hex::encode(zl));
        debug!("ZR {}", hex::encode(zr));

        // kL_new = 8*ZL + kL_parent (no modulo reduction)
        let mut kl = [0u8; 32];
        add_scalar_inplace_mul8(&mut kl, zl, &key[..32]);
        debug!("kl {}", hex::encode(kl));

        // kR_new = ZR + kR_parent
        let mut kr = [0u8; 32];
        add_scalar_inplace(&mut kr, zr, &key[32..]);
        debug!("kr {}", hex::encode(kr));

        // update chain code: HMAC(chain_code, 0x01 || key || index)
        tmp[0] = 0x01;
        tmp[1..65].copy_from_slice(&key);
        let mut mac = HmacSha512::new_from_slice(&chain_code)?;
        mac.update(&tmp[..69]);
        let new_chain = mac.finalize().into_bytes();
        chain_code.copy_from_slice(&new_chain[32..]); // upper 32 bytes
        debug!("cc {}", hex::encode(chain_code));

        // update key
        key[..32].copy_from_slice(&kl);
        key[32..].copy_from_slice(&kr);
    }

    Ok(ExtendedPrivateKey { key, chain_code })
}

/// Add two 32-byte scalars inplace: out = a + b (little-endian)
fn add_scalar_inplace(out: &mut [u8; 32], a: &[u8], b: &[u8]) {
    let mut carry = 0u16;
    for i in 0..32 {
        let sum = a[i] as u16 + b[i] as u16 + carry;
        out[i] = (sum & 0xFF) as u8;
        carry = sum >> 8;
    }
}

/// Add 32-byte scalar a and 32-byte scalar b*8 inplace: out = 8*b + a
fn add_scalar_inplace_mul8(out: &mut [u8; 32], b: &[u8], a: &[u8]) {
    let mut tmp = [0u8; 32];
    tmp[0..28].copy_from_slice(b);
    for _ in 0..3 {
        // multiply by 8
        let mut tmp2 = [0u8; 32];
        add_scalar_inplace(&mut tmp2, &tmp, &tmp);
        tmp.copy_from_slice(&tmp2);
    }
    add_scalar_inplace(out, &tmp, a);
}

pub async fn recover_ledger_seed(mnemonic: &str, aindex: u32) -> Result<ExtendedSpendingKey> {
    let mnemonic = Mnemonic::parse(mnemonic)?;
    let seed = mnemonic.to_seed("");

    let coin_type = MainNetwork.coin_type();
    const H: u32 = 0x8000_0000;
    let path = [0x2C | H, coin_type | H, H, H, H];
    let zip32_seed: [u8; 32] = derive_ed25519_private_key(seed, &path)?;

    // Use ZIP32 derivation with path
    // 32'/85'/account'
    let msk = sapling_crypto::zip32::ExtendedSpendingKey::master(&zip32_seed);
    let extsk = msk
        .derive_child(ChildIndex::hardened(32))
        .derive_child(ChildIndex::hardened(coin_type))
        .derive_child(ChildIndex::hardened(aindex));

    Ok(extsk)
}

fn derive_ed25519_private_key(seed: [u8; 64], path: &[u32]) -> Result<[u8; 32]> {
    let m = ledger_derivation_from_seed(&seed)?;
    let k = derive_hardened_ed25519(&m, path)?;
    let mut child = [0u8; 32];
    child.copy_from_slice(&k.key[..32]);
    Ok(child)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn t() -> Result<()> {
        // let m = Mnemonic::parse("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about")?;
        // let s = m.to_seed("");
        let s = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let m = ledger_derivation_from_seed(s)?;
        let k = derive_hardened_ed25519(&m, &[
            0x8000_0000,
            0x8000_0001,
            0x8000_0002,
            0x8000_0002,
            0x8000_0000 | 1000000000,
            ])?;
        println!("{}", hex::encode(k.key));
        println!("{}", hex::encode(k.chain_code));

        Ok(())
    }
}
