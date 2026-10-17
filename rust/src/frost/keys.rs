//! Group viewing keys for FROST accounts, and the ZIP 2005 derivation.
//!
//! A FROST group has no spending key: `ak` comes out of the DKG, so the rest
//! of the Orchard full viewing key cannot be derived from `sk` the normal
//! way. Two derivations exist, told apart by the account's DKG version:
//!
//! - **Legacy (dkg-0, dkg-1).** `nk` and `rivk` are those of the broadcast
//!   account, derived from its `sk` by the protocol spec; only `ak` is
//!   swapped for the group key. The resulting `rivk` satisfies neither branch
//!   of ZIP 2005's Recovery Statement, so notes at this address are not
//!   quantum recoverable.
//! - **ZIP 2005 (dkg-2).** The `use_qsk` branch: `nk = H^nk(sk)`,
//!   `qsk = truncate_32(PRF^expand_sk([0x0C]))`,
//!   `qk = BLAKE3.derive_key("Zcash ZIP 2005 qk-derivation v1", qsk)`,
//!   `rivk = ToScalar(PRF^expand_qk([0x0D] || ak || nk))`. Only `rivk`, and
//!   therefore the address, differs from the legacy derivation.
//!
//! In both cases `sk` is the Orchard spending key of the broadcast account
//! the group shares, so every member computes the same key. No upstream crate
//! offers the `use_qsk` functions yet; they are implemented here from the
//! BLAKE2b, BLAKE3 and Pallas primitives, and checked against the published
//! `use_qsk` vectors in the tests below.

use anyhow::{Context, Result};
use ff::{FromUniformBytes, PrimeField};
use orchard::keys::{FullViewingKey, SpendingKey};
use pasta_curves::pallas;

/// dkg-0: before message signing; no round-0 key, unsigned messages.
pub const DKG_VERSION_0: u8 = 0;
/// dkg-1: random ed25519 message key, legacy `rivk`.
pub const DKG_VERSION_1: u8 = 1;
/// dkg-2: ZIP 2005 `use_qsk` derivation.
pub const DKG_VERSION_2: u8 = 2;
/// The version new DKGs run.
pub const CURRENT_DKG_VERSION: u8 = DKG_VERSION_2;

/// Context string of `H^qk`, ZIP 2005 § 4.2.3.
pub const QK_CONTEXT: &str = "Zcash ZIP 2005 qk-derivation v1";

/// Whether a version's `rivk` satisfies the Recovery Statement.
pub fn is_recoverable(version: u8) -> bool {
    version >= DKG_VERSION_2
}

/// `PRF^expand_key(t)`: BLAKE2b-512 personalised `Zcash_ExpandSeed` over
/// `key || t`. Protocol spec § 5.4.2.
fn prf_expand(key: &[u8], t: &[&[u8]]) -> [u8; 64] {
    let mut state = blake2b_simd::Params::new()
        .hash_length(64)
        .personal(b"Zcash_ExpandSeed")
        .to_state();
    state.update(key);
    for part in t {
        state.update(part);
    }
    *state.finalize().as_array()
}

/// `H^nk(sk)`, as the bytes of the nullifier deriving key.
pub fn nk(sk: &SpendingKey) -> [u8; 32] {
    let fvk = FullViewingKey::from(sk).to_bytes();
    fvk[32..64].try_into().unwrap()
}

/// `H^qsk(sk) = truncate_32(PRF^expand_sk([0x0C]))`.
pub fn qsk(sk: &SpendingKey) -> [u8; 32] {
    prf_expand(sk.to_bytes(), &[&[0x0C]])[..32]
        .try_into()
        .unwrap()
}

/// `H^qk(qsk) = BLAKE3.derive_key(QK_CONTEXT, qsk, 32)`.
pub fn qk(qsk: &[u8; 32]) -> [u8; 32] {
    blake3::derive_key(QK_CONTEXT, qsk)
}

/// `H^rivk_ext_qk(ak, nk) = ToScalar(PRF^expand_qk([0x0D] || ak || nk))`,
/// as the canonical scalar bytes.
pub fn rivk_ext_qk(qk: &[u8; 32], ak: &[u8; 32], nk: &[u8; 32]) -> [u8; 32] {
    let bytes = prf_expand(qk, &[&[0x0D], ak, nk]);
    pallas::Scalar::from_uniform_bytes(&bytes).to_repr()
}

/// The group full viewing key for a DKG version, from the broadcast
/// account's `sk` and the group `ak` (its 32-byte even-y encoding).
pub fn group_fvk(version: u8, sk: &SpendingKey, ak: &[u8; 32]) -> Result<FullViewingKey> {
    let mut fvkb = FullViewingKey::from(sk).to_bytes();
    fvkb[0..32].copy_from_slice(ak);
    if is_recoverable(version) {
        let nk = nk(sk);
        let rivk = rivk_ext_qk(&qk(&qsk(sk)), ak, &nk);
        fvkb[64..96].copy_from_slice(&rivk);
    }
    FullViewingKey::from_bytes(&fvkb).context("group full viewing key is invalid")
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Vector {
        sk: &'static str,
        ak: &'static str,
        nk: &'static str,
        qsk: &'static str,
        qk: &'static str,
        rivk_ext: &'static str,
    }

    /// The published `use_qsk` vectors (zafe-core `zip2005_use_qsk.json`),
    /// independently recomputed with a standard-library Python script before
    /// being embedded. A change in the ZIP's tags or context string before
    /// NU6.3 fails this test rather than silently moving the address.
    const VECTORS: &[Vector] = &[
        Vector {
            sk: "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
            ak: "262473baab7c8fbf24c39e0b2d2dd4736f7b74d7f3b6d32d1e8c62421368ad0e",
            nk: "415e752227cb69652e2afa94816f630dcec1acdf3c3fb02e1e6b046e12a43111",
            qsk: "3282b2ba8b4e2a167865e5f83cbfe0ba868c7511996646d688c47d0df9d0800e",
            qk: "eb2b2b6e29b2821e638deb0936ac7ee8c67cceb9bc0bda903bffef016a339496",
            rivk_ext: "3961a3868cc3489cf7c6c3b6b619f11c208d38d63f05fb8500bcce5d4c6cef29",
        },
        Vector {
            sk: "202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f",
            ak: "b6c6ed12bf8de427e60a72615e3ef838cd90572bb3a7b87cd92a928e4c959924",
            nk: "c3e108702d8eab6e18289d40ec275ad48723d6f1bd9a0e5bdc35f56d8126820b",
            qsk: "d2f191d5316a447c9b3695168b6a199ea5bfa9d6c1ee538c34cc3e6c72207092",
            qk: "ebee3906d5b8219fb1c88df7717fc12bf70875fa3cb32574e5d2f515e3cd4a76",
            rivk_ext: "bdfd6c44d326a9afe81584d21d8f56977abb12fea2c26f8fa24c22d88e0ffd28",
        },
        Vector {
            sk: "404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f",
            ak: "7199cb8dcad65ac39e07845e6e90dad3db3678d0d94ad5319fb9cc480b799a2c",
            nk: "4dab5080d322b5edbe965f14eb9018cf1ae45fcebd2bec18d0b14aa2b9ca550a",
            qsk: "55c38ef2db8674e006602899e59c9b596a3a443cb5401933a64a9177028e5fe3",
            qk: "c4441905e40aaebd8612340e58937c30d8c5c3facbfdd3a449900b13309f3411",
            rivk_ext: "dbefd1758ed49ad4407e0dd2b7f9817183cef03d2d1c6c089867d02e9d87840e",
        },
        Vector {
            sk: "606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f",
            ak: "a11032f8472dac59c147446a31e9d5216ad3633d23fea457c09687ed20de4e30",
            nk: "e9c1f5e89f6dd09730b34d9c53850061fe0e27fedf1a129d070e55e51c54663f",
            qsk: "4f30b0163bd2e244c2342ba391397fd637888d917e59911c91557308316dbb83",
            qk: "59769600e97271c9abb6e37f42983ec72d3943c18d6b87a007ae5e0ab7c6f992",
            rivk_ext: "14880f506da0342f710da7ebe5f9893aa70882a14fce3de51564dc0e1b756c24",
        },
        Vector {
            sk: "808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9f",
            ak: "bc8e295497b870df6e9245fca673644a573fac6e87c06a359947052a484d4e34",
            nk: "f7d65fb89f857e2d33d780b6f320912ae96b041f59202a413be9ed2b61a00526",
            qsk: "f0b03c8a52c26c8f7c6da17fb39ca102e24bdab578532038442989a1c9ad3e46",
            qk: "2981b127055c8bcfa7e907f54b711b36f960aea9ffd9f3e8ed5056eb11932c53",
            rivk_ext: "5e4d9ac9325c8459f454e965bbfe22e1e74bf00cd1b32dd603c453894e4e5c33",
        },
        Vector {
            sk: "a0a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebf",
            ak: "54fc20a13108675cc4e9c2b3a7db4935c6a5e0ae3b7d06a414e942bb0924f533",
            nk: "ed939edfe0fd34727520bf3fe1c897e64e018f036ac6d72de58e98a8b7fe3102",
            qsk: "11fcf1e2a269d29a054484ed675c4a1d6b373959bf76c7f69f0293fbcbbbe6ed",
            qk: "729120bc0a537024e08c54bf2608fbc0d96c1d1b0282ad45b9b45e96b9f99e2c",
            rivk_ext: "bd4b9f800e66d5dcc36a7433203bbdcfa9b50a68f9200f8d901642ef7833d73d",
        },
        Vector {
            sk: "c0c1c2c3c4c5c6c7c8c9cacbcccdcecfd0d1d2d3d4d5d6d7d8d9dadbdcdddedf",
            ak: "b9c12dae434a642dee6b4f2a69f89b94b2ab5c49c068aeebbe718cbaeacff715",
            nk: "01fd7cfb9766b81d0576747bc26e8c5863ce168fc3de4592c378ad0eb2001f36",
            qsk: "ce0e4e78f8016917aa163de34243c3dcb7b347fd71478e1efc54d18525c1b19d",
            qk: "f28a544e6ecdd91afb0582475015a0110a8b7d7b1f6bb99112b525d810e9c67b",
            rivk_ext: "3dbc80db8723f3309908dcfd77e77b07d4e1987518853a912113d91993217315",
        },
        Vector {
            sk: "e0e1e2e3e4e5e6e7e8e9eaebecedeeeff0f1f2f3f4f5f6f7f8f9fafbfcfdfeff",
            ak: "b305ad406f711aa989d1d6cd4884c9ef363400a1c9710013d91042610c27d139",
            nk: "85aa381d0d90261d2778480d39ca0dd6d7e8c7622f49fcd40427376994c9ed0f",
            qsk: "c275d3b32fc163bcd5b21fdd0f6bea2bb43c6fbc735c3afe3b18d544f997b0ff",
            qk: "bf2f6500f2b79c408cee47e9e803214d3f78eda43b13ff5e28e2f792aac834d7",
            rivk_ext: "f19e0b2847c4c04a2258f7e2822281884460d337dee34a6b101413fff8d1041b",
        },
    ];

    fn bytes32(hex: &str) -> [u8; 32] {
        hex::decode(hex).unwrap().try_into().unwrap()
    }

    fn sk(hex: &str) -> SpendingKey {
        SpendingKey::from_bytes(bytes32(hex)).unwrap()
    }

    #[test]
    fn use_qsk_derivation_matches_the_published_vectors() {
        for (i, v) in VECTORS.iter().enumerate() {
            let sk = sk(v.sk);
            let ak = bytes32(v.ak);
            assert_eq!(ak[31] & 0x80, 0, "vector {i}: ak is not even-y");
            let nk = nk(&sk);
            assert_eq!(hex::encode(nk), v.nk, "vector {i}: nk");
            let qsk = qsk(&sk);
            assert_eq!(hex::encode(qsk), v.qsk, "vector {i}: qsk");
            let qk = qk(&qsk);
            assert_eq!(hex::encode(qk), v.qk, "vector {i}: qk");
            let rivk = rivk_ext_qk(&qk, &ak, &nk);
            assert_eq!(hex::encode(rivk), v.rivk_ext, "vector {i}: rivk_ext");

            let fvk = group_fvk(DKG_VERSION_2, &sk, &ak).unwrap().to_bytes();
            assert_eq!(
                hex::encode(fvk),
                format!("{}{}{}", v.ak, v.nk, v.rivk_ext),
                "vector {i}: fvk"
            );
        }
    }

    /// Versions 0 and 1 keep the broadcast account's `rivk`; version 2
    /// replaces it. `ak` and `nk` are the same in all three, so only the
    /// address changes between a legacy group and its upgraded sibling.
    #[test]
    fn legacy_and_zip2005_builders_share_ak_and_nk_and_differ_in_rivk() {
        let v = &VECTORS[0];
        let sk = sk(v.sk);
        let ak = bytes32(v.ak);
        let legacy0 = group_fvk(DKG_VERSION_0, &sk, &ak).unwrap().to_bytes();
        let legacy1 = group_fvk(DKG_VERSION_1, &sk, &ak).unwrap().to_bytes();
        let zip2005 = group_fvk(DKG_VERSION_2, &sk, &ak).unwrap().to_bytes();
        assert_eq!(legacy0, legacy1);
        assert_eq!(legacy0[..64], zip2005[..64]);
        assert_ne!(legacy0[64..], zip2005[64..]);

        // The legacy builder is what the first DKG shipped: the broadcast
        // account's own FVK with the group `ak` written over its first 32 bytes.
        let mut expected = FullViewingKey::from(&sk).to_bytes();
        expected[..32].copy_from_slice(&ak);
        assert_eq!(legacy0, expected);
        assert!(!is_recoverable(DKG_VERSION_1));
        assert!(is_recoverable(DKG_VERSION_2));
    }
}
