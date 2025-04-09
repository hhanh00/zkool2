use bip32::{ChildNumber, PrivateKey as _};
use fpdec::Decimal;
use pczt::roles::{creator::Creator, updater::Updater};
use rand_core::OsRng;
use ripemd::Ripemd160;
use secp256k1::SecretKey;
use sha2::{Digest as _, Sha256};
use sqlx::{sqlite::SqliteRow, Pool, Row, Sqlite};
use zcash_keys::encoding::AddressCodec as _;
use zcash_primitives::{
    legacy::TransparentAddress,
    transaction::{
        builder::{BuildConfig, Builder},
        fees::zip317::FeeRule,
    }, zip32::fingerprint::SeedFingerprint,
};
use zcash_protocol::{
    consensus::{BlockHeight, Network},
    value::Zatoshis,
};
use zcash_transparent::{bundle::{OutPoint, TxOut}, pczt::Bip32Derivation};

use crate::{
    sync::get_tree_state,
    warp::hasher::{OrchardHasher, SaplingHasher},
    Client,
};

use super::{error::Result, Recipient};

pub async fn prepare(
    network: &Network,
    connection: &Pool<Sqlite>,
    client: &mut Client,
    account: u32,
    _recipients: &[Recipient],
    sender_pay_fees: bool,
    src_pools: u8,
) -> Result<()> {
    let hs = SaplingHasher::default();
    let ho = OrchardHasher::default();

    let sf = SeedFingerprint::from_seed(&[0u8; 32]).unwrap();


    let height = 2200000;
    let (ts, to) = get_tree_state(network, client, height).await?;
    let sapling_anchor = ts.to_edge(&hs).root(&hs);
    let orchard_anchor = to.to_edge(&ho).root(&ho);

    let sks = sqlx::query(
        "SELECT sk, address FROM transparent_address_accounts WHERE account = ? 
        AND scope = 0",
    )
    .bind(account)
    .map(|row: SqliteRow| {
        let sk: Vec<u8> = row.get(0);
        let address: String = row.get(1);
        (sk, address)
    })
    .fetch_all(connection)
    .await
    .map_err(anyhow::Error::from)?;
    let (sk, address) = sks.last().unwrap();
    let sk = SecretKey::from_bytes(&sk.clone().try_into().unwrap()).unwrap();
    let pubkey = sk.public_key(&secp256k1::Secp256k1::new());
    let pkh1 = Ripemd160::digest(Sha256::digest(pubkey.serialize())).to_vec();
    let address = TransparentAddress::decode(network, address).unwrap();
    let TransparentAddress::PublicKeyHash(pkh2) = address else { panic!() };
    assert!(pkh1 == pkh2);

    let mut builder = Builder::new(
        network,
        BlockHeight::from_u32(height),
        BuildConfig::Standard {
            sapling_anchor: sapling_crypto::Anchor::from_bytes(sapling_anchor).into_option(),
            orchard_anchor: orchard::Anchor::from_bytes(orchard_anchor).into_option(),
        },
    );
    builder.add_transparent_input(pubkey, OutPoint::new([0u8; 32], 0), TxOut {
        value: Zatoshis::from_u64(100_010_000).unwrap(),
        script_pubkey: address.script(),
    }).unwrap();
    builder
        .add_transparent_output(
            &TransparentAddress::decode(network, "t1VMifEXtB5iDstRKrpwkxweRmNdrhDZxGk").unwrap(),
            Zatoshis::from_u64(100_000_000).unwrap(),
        )
        .unwrap();
    let pczt_result = builder.build_for_pczt(OsRng, &FeeRule::standard()).unwrap();
    
    let pczt = Creator::build_from_parts(pczt_result.pczt_parts).unwrap();
    let updater = Updater::new(pczt);
    let updater = updater.update_global_with(|mut up| {
        up.set_proprietary("sender_pay_fees".to_string(), vec![sender_pay_fees as u8]);
        up.set_proprietary("pools".to_string(), vec![src_pools]);
    });
    let updater = updater
        .update_transparent_with(|mut up| {
            up.update_input_with(0, |mut inp| {
                let dp = ChildNumber::new(0, true).unwrap();
                let derivation = Bip32Derivation::parse(sf.to_bytes(), vec![dp.into()]).unwrap();
                inp.set_bip32_derivation(pubkey.serialize(), derivation);
                Ok(())
            })?;
            up.update_output_with(0, |mut out| {
                out.set_user_address("t1VMifEXtB5iDstRKrpwkxweRmNdrhDZxGk".to_string());
                Ok(())
            })?;
            Ok(())
        })
        .unwrap();
    let pczt = updater.finish();

    println!("PCZT: {:?}", pczt);

    Ok(())
}

pub fn to_zec(amount: u64) -> String {
    let zats = fpdec::Decimal::from(amount);
    let zec: Decimal = zats / 100_000_000;
    zec.to_string()
}
