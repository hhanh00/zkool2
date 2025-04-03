use anyhow::Result;
use secp256k1::SecretKey;

pub fn export_tsk(sk: &SecretKey) -> String {
    let mut v = sk.secret_bytes().to_vec();
    v.push(0x01);
    bs58::encode(v).with_check_version(0x80).into_string()
}

pub fn import_tsk(tsk: &str) -> Result<SecretKey> {
    let v = bs58::decode(tsk).with_check(Some(0x80)).into_vec()?;
    if v.len() != 34 {
        return Err(anyhow::anyhow!("Invalid TSK length"));
    }
    let sk = SecretKey::from_slice(&v[1..33]).unwrap();
    Ok(sk)
}
