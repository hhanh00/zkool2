use anyhow::Result;
use secp256k1::SecretKey;

pub struct TSecretKey(pub SecretKey, pub bool);

pub fn export_tsk(sk: &TSecretKey) -> String {
    let mut v = sk.0.secret_bytes().to_vec();
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
    let sk = SecretKey::from_slice(&v[1..33]).unwrap();
    let compressed = v.len() == 34 && v[33] == 0x01;
    Ok(TSecretKey(sk, !compressed))
}
