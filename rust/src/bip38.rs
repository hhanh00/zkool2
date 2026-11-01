use anyhow::Result;
use secp256k1::SecretKey;

pub struct TSecretKey(pub SecretKey, pub bool);

pub fn export_tsk(sk: &TSecretKey) -> String {
    let mut v = sk.0.to_secret_bytes().to_vec();
    if !sk.1 {
        v.push(0x01);
    }
    bs58::encode(v).with_check_version(0x80).into_string()
}

pub fn import_tsk(tsk: &str) -> Result<TSecretKey> {
    let v = bs58::decode(tsk).with_check(Some(0x80)).into_vec()?;
    if v.len() != 33 && v.len() != 34 {
        return Err(anyhow::anyhow!("Invalid TSK length {}", v.len()));
    }
    // The only trailing byte WIF defines is 0x01, "compressed"; any other
    // value is not an uncompressed key but a malformed string (#1506).
    if v.len() == 34 && v[33] != 0x01 {
        return Err(anyhow::anyhow!("Invalid TSK compression marker"));
    }
    let sk_bytes: [u8; 32] = v[1..33].try_into()?;
    let sk = SecretKey::from_secret_bytes(sk_bytes).unwrap();
    let compressed = v.len() == 34;
    Ok(TSecretKey(sk, !compressed))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn secret() -> SecretKey {
        SecretKey::from_secret_bytes([7u8; 32]).unwrap()
    }

    /// A WIF over the secret with `marker` as the 34th byte, checksummed.
    fn wif_with_marker(marker: u8) -> String {
        let mut v = secret().to_secret_bytes().to_vec();
        v.push(marker);
        bs58::encode(v).with_check_version(0x80).into_string()
    }

    #[test]
    fn a_compressed_wif_round_trips() {
        let wif = export_tsk(&TSecretKey(secret(), false));
        let imported = import_tsk(&wif).unwrap();
        assert_eq!(imported.0, secret());
        assert!(!imported.1, "the 0x01 marker means compressed");
    }

    #[test]
    fn a_wif_without_a_marker_is_uncompressed() {
        let wif = export_tsk(&TSecretKey(secret(), true));
        let imported = import_tsk(&wif).unwrap();
        assert_eq!(imported.0, secret());
        assert!(imported.1);
    }

    #[test]
    fn a_marker_other_than_0x01_is_refused() {
        let wif = wif_with_marker(0x02);
        let err = match import_tsk(&wif) {
            Ok(_) => panic!("a 0x02 marker was accepted"),
            Err(e) => e.to_string(),
        };
        assert!(err.contains("compression marker"), "{err}");
        assert!(!crate::key::is_valid_transparent_key(&wif));
    }
}
