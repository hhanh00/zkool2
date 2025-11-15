use anyhow::{anyhow, Result};
use bip39::Mnemonic;
use clap::Parser;
use orchard::keys::FullViewingKey;
use sapling_crypto::zip32::DiversifiableFullViewingKey;
use sqlx::{sqlite::SqliteConnectOptions, Connection, Row, SqliteConnection};
use std::str::FromStr;
use zcash_keys::{
    encoding::AddressCodec,
    keys::{UnifiedAddressRequest, UnifiedSpendingKey},
};
use zcash_primitives::zip32::{fingerprint::SeedFingerprint, AccountId};
use zcash_protocol::consensus::{MainNetwork, NetworkUpgrade, Parameters};
use zcash_transparent::keys::{AccountPubKey, NonHardenedChildIndex, TransparentKeyScope};

#[derive(Parser, Debug)]
#[command(name = "Auto Import")]
#[command(about = "Automatically import seed phrase and create multiple accounts", long_about = None)]
struct Args {
    /// Path to the database file
    #[arg(short, long)]
    db_path: String,

    /// Seed phrase (12-24 words)
    #[arg(short, long)]
    seed_phrase: String,

    /// Number of accounts to create
    #[arg(short, long)]
    count: u32,

    /// Starting account index (default: 0)
    #[arg(short = 'i', long, default_value = "0")]
    start_index: u32,

    /// Account name prefix (will be suffixed with index)
    #[arg(short = 'n', long, default_value = "Account")]
    name_prefix: String,

    /// Optional passphrase for seed
    #[arg(short, long, default_value = "")]
    passphrase: String,

    /// Birth height (block height when wallet was created)
    #[arg(short, long)]
    birth: Option<u32>,

    /// Database password (if encrypted)
    #[arg(long, default_value = "")]
    db_password: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Validate seed phrase
    let mnemonic = Mnemonic::from_str(&args.seed_phrase)
        .map_err(|_| anyhow!("Invalid seed phrase"))?;

    println!("✓ Seed phrase validated");
    println!("Database: {}", args.db_path);
    println!("Creating {} accounts starting from index {}", args.count, args.start_index);

    // Connect to database
    let mut options = SqliteConnectOptions::new().filename(&args.db_path);
    if !args.db_password.is_empty() {
        options = options.pragma("key", args.db_password.clone());
    }

    let mut connection = SqliteConnection::connect_with(&options).await?;
    println!("✓ Connected to database");

    let network = MainNetwork;
    let seed = mnemonic.to_seed(&args.passphrase);
    let seed_fingerprint = SeedFingerprint::from_seed(&seed).unwrap().to_bytes();

    // Determine birth height
    let birth = args.birth.unwrap_or_else(|| {
        network
            .activation_height(NetworkUpgrade::Sapling)
            .unwrap()
            .into()
    });

    // Create accounts
    for i in 0..args.count {
        let aindex = args.start_index + i;
        let account_name = format!("{} {}", args.name_prefix, aindex);

        println!("\n[{}/{}] Creating account: {}", i + 1, args.count, account_name);

        match create_account(
            &mut connection,
            &network,
            &account_name,
            &args.seed_phrase,
            &args.passphrase,
            &seed,
            &seed_fingerprint,
            aindex,
            birth,
        ).await {
            Ok(account_id) => {
                println!("  ✓ Account created with ID: {}", account_id);
            }
            Err(e) => {
                eprintln!("  ✗ Failed to create account: {}", e);
                continue;
            }
        }
    }

    println!("\n✓ Done! Created {} accounts", args.count);

    Ok(())
}

async fn create_account(
    connection: &mut SqliteConnection,
    network: &MainNetwork,
    name: &str,
    seed_phrase: &str,
    passphrase: &str,
    seed: &[u8],
    seed_fingerprint: &[u8],
    aindex: u32,
    birth: u32,
) -> Result<u32> {
    let mut tx = connection.begin().await?;

    // Store account metadata
    let (last_position,): (u32,) = sqlx::query_as("SELECT COALESCE(MAX(position), 0) FROM accounts")
        .fetch_one(&mut *tx)
        .await?;

    let account_id: u32 = sqlx::query(
        "INSERT INTO accounts(name, seed_fingerprint, birth, aindex, dindex, def_dindex, position, use_internal, saved, hidden, internal)
        VALUES (?, ?, ?, 0, 0, 0, ?, FALSE, FALSE, FALSE, FALSE)
        RETURNING id_account"
    )
    .bind(name)
    .bind(seed_fingerprint)
    .bind(birth)
    .bind(last_position + 1)
    .map(|row: sqlx::sqlite::SqliteRow| row.get(0))
    .fetch_one(&mut *tx)
    .await?;

    // Store seed
    sqlx::query(
        "UPDATE accounts
         SET seed = ?,
             passphrase = ?,
             seed_fingerprint = ?,
             aindex = ?
         WHERE id_account = ?"
    )
    .bind(seed_phrase)
    .bind(passphrase)
    .bind(seed_fingerprint)
    .bind(aindex)
    .bind(account_id)
    .execute(&mut *tx)
    .await?;

    // Generate UnifiedSpendingKey
    let usk = UnifiedSpendingKey::from_seed(
        network,
        seed,
        AccountId::try_from(aindex).unwrap(),
    )?;

    let uvk = usk.to_unified_full_viewing_key();
    let (_, di) = uvk.default_address(UnifiedAddressRequest::AllAvailableKeys)?;
    let dindex: u32 = di.try_into()?;

    // Initialize transparent account
    sqlx::query("INSERT INTO transparent_accounts(account) VALUES (?)")
        .bind(account_id)
        .execute(&mut *tx)
        .await?;

    let activation_height_t: u32 = 0;
    sqlx::query("INSERT OR REPLACE INTO sync_heights(account, pool, height) VALUES (?, 0, ?)")
        .bind(account_id)
        .bind(birth.max(activation_height_t))
        .execute(&mut *tx)
        .await?;

    // Store transparent keys
    let tsk = usk.transparent();
    let tsk_bytes = tsk.to_bytes();
    sqlx::query("UPDATE transparent_accounts SET xsk = ? WHERE account = ?")
        .bind(tsk_bytes.as_slice())
        .bind(account_id)
        .execute(&mut *tx)
        .await?;

    let tvk = tsk.to_account_pubkey();
    let tvk_bytes = tvk.serialize();
    sqlx::query("UPDATE transparent_accounts SET xvk = ? WHERE account = ?")
        .bind(tvk_bytes.as_slice())
        .bind(account_id)
        .execute(&mut *tx)
        .await?;

    // Derive transparent address
    let scope = TransparentKeyScope::EXTERNAL;
    let tpk = tvk
        .derive_address_pubkey(scope, NonHardenedChildIndex::from_index(dindex).unwrap())
        .unwrap()
        .serialize();

    use ripemd::Digest as _;
    use sha2::Sha256;
    let pkh: [u8; 20] = ripemd::Ripemd160::digest(Sha256::digest(&tpk)).into();
    use zcash_primitives::legacy::TransparentAddress;
    let taddr = TransparentAddress::PublicKeyHash(pkh);
    let taddr_str = taddr.encode(network);

    // Derive transparent secret key
    let tsk_derived = tsk
        .derive_secret_key(scope, NonHardenedChildIndex::from_index(dindex).unwrap())
        .unwrap();
    let tsk_bytes = tsk_derived.secret_bytes().to_vec();

    sqlx::query(
        "INSERT INTO transparent_address_accounts(account, scope, dindex, sk, pk, address)
        VALUES (?, 0, ?, ?, ?, ?)"
    )
    .bind(account_id)
    .bind(dindex)
    .bind(tsk_bytes.as_slice())
    .bind(tpk.as_slice())
    .bind(&taddr_str)
    .execute(&mut *tx)
    .await?;

    // Initialize sapling account
    sqlx::query("INSERT INTO sapling_accounts(account, xvk) VALUES (?, '')")
        .bind(account_id)
        .execute(&mut *tx)
        .await?;

    let activation_height_s: u32 = network
        .activation_height(NetworkUpgrade::Sapling)
        .unwrap()
        .into();
    sqlx::query("INSERT OR REPLACE INTO sync_heights(account, pool, height) VALUES (?, 1, ?)")
        .bind(account_id)
        .bind(birth.max(activation_height_s))
        .execute(&mut *tx)
        .await?;

    // Store sapling keys
    let sxsk = usk.sapling();
    let sxsk_bytes = sxsk.to_bytes();
    sqlx::query("UPDATE sapling_accounts SET xsk = ? WHERE account = ?")
        .bind(sxsk_bytes.as_slice())
        .bind(account_id)
        .execute(&mut *tx)
        .await?;

    let sxvk = sxsk.to_diversifiable_full_viewing_key();
    let saddr = sxvk.address(dindex.into()).unwrap();
    let saddr_str = saddr.encode(network);

    sqlx::query("UPDATE sapling_accounts SET xvk = ?, address = ? WHERE account = ?")
        .bind(sxvk.to_bytes().as_slice())
        .bind(&saddr_str)
        .bind(account_id)
        .execute(&mut *tx)
        .await?;

    // Initialize orchard account
    sqlx::query("INSERT INTO orchard_accounts(account, xvk) VALUES (?, '')")
        .bind(account_id)
        .execute(&mut *tx)
        .await?;

    let activation_height_o: u32 = network
        .activation_height(NetworkUpgrade::Nu5)
        .unwrap()
        .into();
    sqlx::query("INSERT OR REPLACE INTO sync_heights(account, pool, height) VALUES (?, 2, ?)")
        .bind(account_id)
        .bind(birth.max(activation_height_o))
        .execute(&mut *tx)
        .await?;

    // Store orchard keys
    let oxsk = usk.orchard();
    let oxsk_bytes = oxsk.to_bytes();
    sqlx::query("UPDATE orchard_accounts SET xsk = ? WHERE account = ?")
        .bind(oxsk_bytes.as_slice())
        .bind(account_id)
        .execute(&mut *tx)
        .await?;

    let oxvk = FullViewingKey::from(oxsk);
    let oxvk_bytes = oxvk.to_bytes();
    sqlx::query("UPDATE orchard_accounts SET xvk = ? WHERE account = ?")
        .bind(oxvk_bytes.as_slice())
        .bind(account_id)
        .execute(&mut *tx)
        .await?;

    // Update dindex
    sqlx::query("UPDATE accounts SET dindex = ?, def_dindex = ? WHERE id_account = ?")
        .bind(dindex)
        .bind(dindex)
        .bind(account_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(account_id)
}
