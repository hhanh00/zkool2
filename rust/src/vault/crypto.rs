use std::collections::HashMap;
use std::io::{Read, Write};

use anyhow::{Result, anyhow};
use argon2::{Algorithm, Argon2, Params, Version};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use chacha20poly1305::{ChaCha20Poly1305, Key, KeyInit, Nonce, aead::Aead};
use rand_core::{OsRng, RngCore};
use x25519_dalek::{PublicKey, StaticSecret};

use crate::api::vault::RestoredAccount;

pub struct AccountPayload {
    pub timestamp: u32,
    pub name: String,
    pub entropy: [u8; 32],
    pub aindex: u32,
    pub use_internal: bool,
    pub birth_height: u32,
}

impl AccountPayload {
    /// Write: name_len(2 BE) | name(var) | entropy(32) | aindex(4 BE) | use_internal(1) | birth_height(4 BE)
    fn write_to<W: Write>(&self, mut w: W) -> Result<()> {
        w.write_u32::<BigEndian>(self.timestamp)?;
        let name_bytes = self.name.as_bytes();
        w.write_u16::<BigEndian>(name_bytes.len() as u16)?;
        w.write_all(name_bytes)?;
        w.write_all(&self.entropy)?;
        w.write_u32::<BigEndian>(self.aindex)?;
        w.write_u8(self.use_internal as u8)?;
        w.write_u32::<BigEndian>(self.birth_height)?;
        Ok(())
    }

    fn read_from<R: Read>(mut r: R) -> Result<Self> {
        let timestamp = r.read_u32::<BigEndian>()?;
        let name_len = r.read_u16::<BigEndian>()? as usize;
        let mut name_buf = vec![0u8; name_len];
        r.read_exact(&mut name_buf)?;
        let name = String::from_utf8(name_buf)
            .map_err(|e| anyhow!("Invalid name: {}", e))?;
        let mut entropy = [0u8; 32];
        r.read_exact(&mut entropy)?;
        let aindex = r.read_u32::<BigEndian>()?;
        let use_internal = r.read_u8()? != 0;
        let birth_height = r.read_u32::<BigEndian>()?;
        Ok(Self { timestamp, name, entropy, aindex, use_internal, birth_height })
    }
}

enum LogEntry {
    Init {
        pk: [u8; 32],                     // X25519 public key, plaintext
        master_key_protected_sk: [u8; 60], // nonce(12) || ciphertext+tag(48), ChaCha20-Poly1305(SK, MasterKey)
        argon2_salt: [u8; 16],            // random salt for Argon2id, generated at initialization
    },
    AddDevice {
        device_id: [u8; 20], // RIPEMD-160 hash, stable per (device, app) pair
        prf_key_protected_sk: [u8; 60], // nonce(12) || ciphertext+tag(48), ChaCha20-Poly1305(SK, PRFKey)
    },
    Account {
        ephemeral_pk: [u8; 32],
        nonce: [u8; 12],
        ciphertext: Vec<u8>, // ChaCha20-Poly1305(serialized AccountPayload, sk), variable due to name
    },
}

impl LogEntry {
    /// Write: type(1) | data
    /// Init:      type(1) | pk(32) | protected_sk(60) | salt(16) = 109
    /// AddDevice: type(1) | device_id(20) | protected_sk(60) = 81
    /// Account:   type(1) | len(2 BE) | ephemeral_pk(32) | nonce(12) | ciphertext(var)
    fn write_to<W: Write>(&self, mut w: W) -> Result<()> {
        match self {
            LogEntry::Init { pk, master_key_protected_sk, argon2_salt } => {
                w.write_u8(0)?;
                w.write_all(pk)?;
                w.write_all(master_key_protected_sk)?;
                w.write_all(argon2_salt)?;
            }
            LogEntry::AddDevice { device_id, prf_key_protected_sk } => {
                w.write_u8(1)?;
                w.write_all(device_id)?;
                w.write_all(prf_key_protected_sk)?;
            }
            LogEntry::Account { ephemeral_pk, nonce, ciphertext } => {
                w.write_u8(2)?;
                w.write_u16::<BigEndian>((32 + 12 + ciphertext.len()) as u16)?;
                w.write_all(ephemeral_pk)?;
                w.write_all(nonce)?;
                w.write_all(ciphertext)?;
            }
        }
        Ok(())
    }

    fn read_from<R: Read>(mut r: R) -> Result<Self> {
        let tag = r.read_u8()?;
        match tag {
            0 => {
                let mut pk = [0u8; 32];
                r.read_exact(&mut pk)?;
                let mut master_key_protected_sk = [0u8; 60];
                r.read_exact(&mut master_key_protected_sk)?;
                let mut argon2_salt = [0u8; 16];
                r.read_exact(&mut argon2_salt)?;
                Ok(LogEntry::Init { pk, master_key_protected_sk, argon2_salt })
            }
            1 => {
                let mut device_id = [0u8; 20];
                r.read_exact(&mut device_id)?;
                let mut prf_key_protected_sk = [0u8; 60];
                r.read_exact(&mut prf_key_protected_sk)?;
                Ok(LogEntry::AddDevice { device_id, prf_key_protected_sk })
            }
            2 => {
                let payload_len = (r.read_u16::<BigEndian>()?) as usize;
                let mut ephemeral_pk = [0u8; 32];
                r.read_exact(&mut ephemeral_pk)?;
                let mut nonce = [0u8; 12];
                r.read_exact(&mut nonce)?;
                let ct_len = payload_len - 32 - 12;
                let mut ciphertext = vec![0u8; ct_len];
                r.read_exact(&mut ciphertext)?;
                Ok(LogEntry::Account { ephemeral_pk, nonce, ciphertext })
            }
            _ => Err(anyhow!("Invalid LogEntry tag: {}", tag)),
        }
    }
}

fn derive_key_from_password(password: &str, salt: &[u8; 16]) -> Result<[u8; 32]> {
    let params = Params::new(
        64 * 1024, // 64 MB in KiB
        3,         // iterations
        2,         // parallelism
        Some(32),  // output length
    ).map_err(|e| anyhow!("Argon2 params failed: {}", e))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0u8; 32];
    argon2.hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| anyhow!("Argon2 hashing failed: {}", e))?;
    Ok(key)
}

pub fn derive_master_key(password: &str) -> Result<Vec<u8>> {
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);

    let master_key = derive_key_from_password(password, &salt)?;

    let sk = StaticSecret::random_from_rng(OsRng);
    let pk = PublicKey::from(&sk);

    // encrypt sk with master_key
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&master_key));
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, sk.as_bytes() as &[u8])
        .map_err(|e| anyhow!("Encryption failed: {}", e))?;

    let mut master_key_protected_sk = [0u8; 60];
    master_key_protected_sk[..12].copy_from_slice(&nonce_bytes);
    master_key_protected_sk[12..].copy_from_slice(&ciphertext);

    let entry = LogEntry::Init {
        pk: *pk.as_bytes(),
        master_key_protected_sk,
        argon2_salt: salt,
    };

    let mut buf = Vec::with_capacity(109);
    entry.write_to(&mut buf)?;
    Ok(buf)
}

pub fn register_device(
    init_bytes: &[u8],
    master_password: &str,
    device_id_str: &str,
    prf_output: [u8; 32],
) -> Result<Vec<u8>> {
    use ripemd::Ripemd160;
    use sha2::Digest;

    // 1. Parse the Init entry from init_bytes
    let mut cursor = std::io::Cursor::new(init_bytes);
    let init_entry = LogEntry::read_from(&mut cursor)?;
    let (master_key_protected_sk, salt) = match &init_entry {
        LogEntry::Init { pk: _, master_key_protected_sk, argon2_salt } => {
            (*master_key_protected_sk, *argon2_salt)
        }
        _ => return Err(anyhow!("Expected Init entry")),
    };

    // 2. Decrypt SK using master password
    let master_key = derive_key_from_password(master_password, &salt)?;
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&master_key));
    let nonce = Nonce::from_slice(&master_key_protected_sk[..12]);
    let sk_bytes = cipher.decrypt(nonce, &master_key_protected_sk[12..] as &[u8])
        .map_err(|e| anyhow!("Wrong vault password: {}", e))?;

    // 3. Hash device_id string to 20 bytes (RIPEMD-160)
    let mut hasher = Ripemd160::new();
    hasher.update(device_id_str.as_bytes());
    let device_id: [u8; 20] = hasher.finalize().into();

    // 4. Derive key from PRF output
    let prf_key = blake2b_simd::Params::new()
        .hash_length(32)
        .personal(b"zkool-vault-prf")
        .to_state()
        .update(&prf_output)
        .finalize();

    // 5. Encrypt SK with PRF-derived key
    let cipher = ChaCha20Poly1305::new(Key::from_slice(prf_key.as_bytes()));
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, sk_bytes.as_slice() as &[u8])
        .map_err(|e| anyhow!("Encryption failed: {}", e))?;

    let mut prf_key_protected_sk = [0u8; 60];
    prf_key_protected_sk[..12].copy_from_slice(&nonce_bytes);
    prf_key_protected_sk[12..].copy_from_slice(&ciphertext);

    // 6. Create AddDevice entry
    let entry = LogEntry::AddDevice { device_id, prf_key_protected_sk };
    let mut buf = Vec::with_capacity(81);
    entry.write_to(&mut buf)?;
    Ok(buf)
}

pub fn encrypt_account(account: AccountPayload, pk: PublicKey) -> Result<Vec<u8>> {
    let mut plaintext = Vec::new();
    account.write_to(&mut plaintext)?;

    let ephemeral_sk = StaticSecret::random_from_rng(OsRng);
    let ephemeral_pk = PublicKey::from(&ephemeral_sk);
    let shared_secret = ephemeral_sk.diffie_hellman(&pk);

    let derived_key = blake2b_simd::Params::new()
        .hash_length(32)
        .key(shared_secret.as_bytes())
        .personal(b"zkool-vault-acct")
        .to_state()
        .finalize();
    let cipher = ChaCha20Poly1305::new(Key::from_slice(derived_key.as_bytes()));
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, plaintext.as_slice() as &[u8])
        .map_err(|e| anyhow!("Encryption failed: {}", e))?;

    let entry = LogEntry::Account {
        ephemeral_pk: *ephemeral_pk.as_bytes(),
        nonce: nonce_bytes,
        ciphertext,
    };

    let mut buf = Vec::new();
    entry.write_to(&mut buf)?;
    Ok(buf)
}

pub fn recover(vault_bytes: &[u8], master_password: &str) -> Result<Vec<RestoredAccount>> {
    let entries = parse_entries(vault_bytes)?;

    let (_, master_key_protected_sk, salt) = entries.iter().find_map(|e| match e {
        LogEntry::Init { pk, master_key_protected_sk, argon2_salt } => {
            Some((*pk, *master_key_protected_sk, *argon2_salt))
        }
        _ => None,
    }).ok_or_else(|| anyhow!("No Init LogEntry found"))?;

    // Derive master key from password + salt
    let master_key = derive_key_from_password(master_password, &salt)?;

    // Decrypt sk: nonce is first 12 bytes, ciphertext+tag is remaining 48 bytes
    let sk_bytes = decrypt_sk(&master_key, &master_key_protected_sk)?;

    tracing::info!("Recovered sk via master password ({} bytes)", sk_bytes.len());

    decrypt_accounts(&entries, &sk_bytes)
}

pub fn recover_with_prf(
    vault_bytes: &[u8],
    device_id_str: &str,
    prf_output: [u8; 32],
) -> Result<Vec<RestoredAccount>> {
    use ripemd::Ripemd160;
    use sha2::Digest;

    let entries = parse_entries(vault_bytes)?;

    // Hash device_id
    let mut hasher = Ripemd160::new();
    hasher.update(device_id_str.as_bytes());
    let device_id: [u8; 20] = hasher.finalize().into();

    // Derive key from PRF output
    let prf_key = blake2b_simd::Params::new()
        .hash_length(32)
        .personal(b"zkool-vault-prf")
        .to_state()
        .update(&prf_output)
        .finalize();

    // Find matching AddDevice entry and try to decrypt SK
    let sk_bytes = entries.iter().find_map(|e| match e {
        LogEntry::AddDevice { device_id: did, prf_key_protected_sk } if *did == device_id => {
            decrypt_sk(prf_key.as_bytes(), prf_key_protected_sk).ok()
        }
        _ => None,
    }).ok_or_else(|| anyhow!("No matching device entry found"))?;

    tracing::info!("Recovered sk via PRF ({} bytes)", sk_bytes.len());

    decrypt_accounts(&entries, &sk_bytes)
}

fn parse_entries(vault_bytes: &[u8]) -> Result<Vec<LogEntry>> {
    let mut entries = Vec::new();
    let mut cursor = std::io::Cursor::new(vault_bytes);
    while cursor.position() < vault_bytes.len() as u64 {
        entries.push(LogEntry::read_from(&mut cursor)?);
    }
    Ok(entries)
}

fn decrypt_sk(key: &[u8], protected_sk: &[u8; 60]) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    let nonce = Nonce::from_slice(&protected_sk[..12]);
    cipher.decrypt(nonce, &protected_sk[12..] as &[u8])
        .map_err(|e| anyhow!("Decryption failed: {}", e))
}

fn decrypt_accounts(entries: &[LogEntry], sk_bytes: &[u8]) -> Result<Vec<RestoredAccount>> {
    let sk = StaticSecret::from(<[u8; 32]>::try_from(sk_bytes).map_err(|_| anyhow!("Invalid sk length"))?);

    let mut deduped: HashMap<([u8; 32], u32), RestoredAccount> = HashMap::new();
    for entry in entries {
        if let LogEntry::Account { ephemeral_pk, nonce, ciphertext } = entry {
            let ephemeral_pk = PublicKey::from(*ephemeral_pk);
            let shared_secret = sk.diffie_hellman(&ephemeral_pk);

            let derived_key = blake2b_simd::Params::new()
                .hash_length(32)
                .key(shared_secret.as_bytes())
                .personal(b"zkool-vault-acct")
                .to_state()
                .finalize();

            let cipher = ChaCha20Poly1305::new(Key::from_slice(derived_key.as_bytes()));
            let nonce = Nonce::from_slice(nonce);
            let plaintext = cipher.decrypt(nonce, ciphertext.as_slice() as &[u8])
                .map_err(|e| anyhow!("Account decryption failed: {}", e))?;

            let account = AccountPayload::read_from(plaintext.as_slice())?;
            let mnemonic = bip39::Mnemonic::from_entropy(&account.entropy)?;
            tracing::info!("Recovered account: name={}, aindex={}, use_internal={}, birth_height={}",
                account.name, account.aindex, account.use_internal, account.birth_height);

            // TODO: Insert only if new and newer than existing entry
            deduped.insert((account.entropy, account.aindex), RestoredAccount {
                name: account.name,
                seed: mnemonic.to_string(),
                aindex: account.aindex,
                use_internal: account.use_internal,
                birth_height: account.birth_height,
            });
        }
    }

    Ok(deduped.into_values().collect())
}
