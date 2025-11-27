use bech32::{Bech32m, Hrp};
use byteorder::LE;
use pczt::{
    roles::{
        low_level_signer::Signer, spend_finalizer::SpendFinalizer,
        tx_extractor::TransactionExtractor,
    },
    Pczt,
};
use secp256k1::{ecdsa::Signature, PublicKey};
use sqlx::{Acquire, SqlitePool};
use std::{fs::File, io::BufReader};
use zcash_keys::encoding::AddressCodec as _;
use zcash_primitives::legacy::{Script, TransparentAddress};

use sapling_crypto::{keys::FullViewingKey, Diversifier, PaymentAddress};
use zcash_address::unified::{self, Encoding, Ufvk};
use zcash_protocol::consensus::MainNetwork;

use crate::{
    api::pay::PcztPackage,
    api::coin::Network,
    ledger::{
        hashers::{
            create_hasher, header_hasher, orchard_hasher, output_hasher, prevout_hasher,
            sequence_hasher, spend_hasher, transparent_hasher, zoutput_hasher,
        },
        APDUCommand,
    },
    IntoAnyhow as _,
};

use super::*;

#[tokio::test]
pub async fn get_device_info() -> LedgerResult<()> {
    let ledger = LEDGER_ZEMU.lock().await.clone().unwrap();
    let res = ledger
        .long_execute(
            &APDUCommand {
                cla: 0x85,
                ins: 0x00,
                p1: 0,
                p2: 0,
                data: vec![],
            },
            &[vec![]],
        )
        .await?;
    assert_eq!(res.retcode, 0x9000);
    println!("{}", hex::encode(&res.data));
    Ok(())
}

#[tokio::test]
pub async fn get_taddress() -> LedgerResult<()> {
    let get_address = APDUCommand {
        cla: 0x85,
        ins: 0x01,
        p1: 0,
        p2: 0,
        data: vec![],
    };
    let mut data = vec![];
    data.write_u32::<LE>(44 | 0x80000000)?;
    data.write_u32::<LE>(133 | 0x80000000)?;
    data.write_u32::<LE>(0x80000000)?;
    data.write_u32::<LE>(0)?;
    data.write_u32::<LE>(0)?;
    let ledger = LEDGER_ZEMU.lock().await.clone().unwrap();
    let res = ledger.long_execute(&get_address, &[data]).await?;
    assert_eq!(res.retcode, 0x9000);
    let pk = &res.data[0..33];
    println!("{}", hex::encode(pk));
    let address = &res.data[33..];
    let pk = bech32::encode::<Bech32m>(Hrp::parse_unchecked("zpk"), pk).anyhow()?;
    println!("{pk}");
    let address = String::from_utf8(address.to_vec()).anyhow()?;
    println!("{address}");
    assert_eq!(address, "t1h31WzbruQhnwHg4XDJ5anLM7CAtwjXmPt");
    Ok(())
}

#[tokio::test]
pub async fn get_fvk() -> LedgerResult<()> {
    // this test will fail with "Inner Ledger error" the first time it runs
    // because user needs to confirm on the device
    // Run it once, then go to the web ui and confirm the operation
    // Run it again and it should pass
    let ledger = LEDGER_ZEMU.lock().await.clone().unwrap();
    let res = ledger
        .execute(APDUCommand {
            cla: 0x85,
            ins: 0xF3,
            p1: 1,
            p2: 0,
            data: 0x80000000u32.to_le_bytes().to_vec(),
        })
        .await?;
    assert_eq!(res.retcode, 0x9000);
    let fvk = hex::encode(&res.data);
    println!("{fvk}");
    assert_eq!(fvk, "d17091f057e2d641328172642f06f821893a564ec8ab98fdd4ca462b8791de5c788c96b31e5e476e954c1a18bd4f1278358924ec9a22d096fe3954d815e353605940cfcf8388fb5e54ebc6f1c9f75a5eddf35227e3d1c4ef003e6f64cd7672db");
    Ok(())
}

#[tokio::test]
pub async fn get_address() -> LedgerResult<()> {
    let ledger = LEDGER_ZEMU.lock().await.clone().unwrap();
    let res = ledger
        .execute(APDUCommand {
            cla: 0x85,
            ins: 0x11,
            p1: 0,
            p2: 0,
            data: 0x80000000u32.to_le_bytes().to_vec(),
        })
        .await?;
    assert_eq!(res.retcode, 0x9000);
    let address = hex::encode(&res.data);
    println!("{address}");
    assert_eq!(address, "a7b6aa86c0c01e5cb4c2285e1d5226c4121687100e3ada3ad60c420516ecc7aeae321e74f1db380bfea40b7a733135376d3234706b71637130396564787a397030703635337863736670647063737063616435776b6b70337071323968766337683275767337776e63616b7771746c366a716b786e39333970");
    Ok(())
}

#[test]
fn payment_address() -> LedgerResult<()> {
    let address_hex = hex::decode("a7b6aa86c0c01e5cb4c2285e1d5226c4121687100e3ada3ad60c420516ecc7aeae321e74f1db380bfea40b7a733135376d3234706b71637130396564787a397030703635337863736670647063737063616435776b6b70337071323968766337683275767337776e63616b7771746c366a716b786e39333970").anyhow()?;
    let address = &address_hex[0..43];
    let div = hex::encode(&address_hex[0..11]);
    println!("{div}");
    let pa = PaymentAddress::from_bytes(&address.try_into().unwrap()).unwrap();
    println!("{}", pa.encode(&MainNetwork));
    let address = String::from_utf8(address_hex[43..].to_vec()).anyhow()?;
    println!("{address}");
    assert_eq!(
        address,
        "zs157m24pkqcq09edxz9p0p653xcsfpdpcspcad5wkkp3pq29hvc7h2uvs7wncakwqtl6jqkxn939p"
    );
    Ok(())
}

#[test]
pub fn ufvk() -> LedgerResult<()> {
    let fvk = hex::decode("de514bb8eba2793731926578513d8ea724d1e4b21fcf8a53b7236711a27ba7bf05eda7736c88143790f66a1793f117100b8b7d0c60c115ee7a2d0e189c4fb416a5077c6d42e7c18de0353751b361a55e90fccbbef3d12fafba1d43a4367feefa").anyhow()?;
    let sapfvk = FullViewingKey::read(&*fvk)?;
    let div: [u8; 11] = hex::decode("a7b6aa86c0c01e5cb4c228")
        .anyhow()?
        .try_into()
        .unwrap();
    let pa = sapfvk.vk.to_payment_address(Diversifier(div)).unwrap();
    println!("{}", pa.encode(&MainNetwork));
    let mut dk = [42u8; 128]; // arbitrary dk because we don't know the real one from the Ledger
    dk[0..96].clone_from_slice(&fvk);
    let sfvk = unified::Fvk::Sapling(dk);
    let ufvk = Ufvk::try_from_items(vec![sfvk]).anyhow()?;
    let ufvk = ufvk.encode(&zcash_protocol::consensus::NetworkType::Main);
    println!("{ufvk}");
    assert_eq!(ufvk, "uview1hytkw2afs80j0zvj3w0nutqs58rpf2qvuygw47k3qmjqj8w9vlwxjp00dpk8tfp6e5jdq0zavetsu5jugxpsqqwssjeh9lxsnugenctuyjhf6my639pv7agspcsvmgk5upj2zjkwm3u98h807sdj5dkvtrle5x2uajl6gzj4ryhuz0sfm2j3g95hm6az2an4tknu0yecmefsrrwqxv6fxgwqpf44awj6fnrhxlytcut20faw");
    Ok(())
}

#[tokio::test]
pub async fn sign_transparent() -> LedgerResult<()> {
    let stage = 2;
    let network = Network::Main;
    let pool = SqlitePool::connect("ledger.db").await.anyhow()?;
    let mut connection = pool.acquire().await.anyhow()?;
    let connection = connection.acquire().await.anyhow()?;
    let mut db_tx = connection.begin().await.anyhow()?;
    let account = 1;
    let s_account = 2;

    let file = File::open("t2s.bin")?;
    let package = bincode::decode_from_reader::<PcztPackage, _, _>(
        BufReader::new(file),
        bincode::config::legacy(),
    )
    .anyhow()?;
    let pczt = Pczt::parse(&package.pczt).unwrap();
    let (pk, address): (Vec<u8>, String) =
        sqlx::query_as("SELECT pk, address FROM transparent_address_accounts WHERE account = ?1")
            .bind(account)
            .fetch_one(&mut *db_tx)
            .await
            .anyhow()?;
    let pk = PublicKey::from_slice(&pk).anyhow()?;
    let address = TransparentAddress::decode(&network, &address).anyhow()?;

    let (xvk,): (Vec<u8>,) = sqlx::query_as("SELECT xvk FROM sapling_accounts WHERE account = ?1")
        .bind(s_account)
        .fetch_one(&mut *db_tx)
        .await
        .anyhow()?;
    let xvk = FullViewingKey::read(&*xvk)?;
    let ovk = xvk.ovk;

    let mut buffers = vec![];
    buffers.push(vec![]);

    let mut data = vec![];
    data.write_u8(pczt.transparent().inputs().len() as u8)?;
    data.write_u8(pczt.transparent().outputs().len() as u8)?;
    data.write_u8(pczt.sapling().spends().len() as u8)?;
    data.write_u8(pczt.sapling().outputs().len() as u8)?;
    buffers.push(data);

    println!(
        "{} {}",
        pczt.transparent().inputs().len(),
        pczt.transparent().outputs().len()
    );
    println!(
        "{} {}",
        pczt.sapling().spends().len(),
        pczt.sapling().outputs().len()
    );

    let tbundle = pczt.transparent();
    for tin in tbundle.inputs() {
        let mut data = vec![];
        data.write_u32::<LE>(44 + 0x80000000)?;
        data.write_u32::<LE>(133 + 0x80000000)?;
        data.write_u32::<LE>(0x80000000)?;
        data.write_u32::<LE>(0)?; // scope
        data.write_u32::<LE>(0)?; // dindex
        address.script().write(&mut data)?;
        data.write_u64::<LE>(*tin.value())?;
        assert_eq!(data.len(), 54);
        buffers.push(data);
    }

    for tout in tbundle.outputs() {
        let mut data = vec![];
        Script(tout.script_pubkey().to_vec()).write(&mut data)?;
        data.write_u64::<LE>(*tout.value())?;
        assert_eq!(data.len(), 34);
        buffers.push(data);
    }

    let sbundle = pczt.sapling();
    for sout in sbundle.outputs() {
        let mut data = vec![];
        let recipient = sout.recipient().expect("Must have a recipient");
        data.write_all(&recipient)?;
        data.write_u64::<LE>(sout.value().expect("Must have value"))?;
        data.write_u8(0xF6)?; // Memo type
        data.write_u8(0x01)?; // Have OVK
        data.write_all(&ovk.0)?;
        assert_eq!(data.len(), 85);
        buffers.push(data);
    }

    let init_tx = APDUCommand {
        cla: 0x85,
        ins: 0xA0,
        p1: 0,
        p2: 5,
        data: vec![],
    };

    let ledger = LEDGER_ZEMU.lock().await.clone().unwrap();
    if stage == 1 {
        let total_len = buffers.iter().map(|b| b.len()).sum::<usize>();
        assert_eq!(
            total_len,
            4 + 54 * tbundle.inputs().len()
                + 34 * tbundle.outputs().len()
                + 85 * sbundle.outputs().len()
        );
        let res = ledger.long_execute(&init_tx, &buffers).await?;
        assert_eq!(res.retcode, 0x9000);
    }

    let mut buffers = vec![];
    buffers.push(vec![]);
    for tin in tbundle.inputs() {
        let mut data = vec![];
        data.write_all(tin.prevout_txid())?;
        data.write_u32::<LE>(*tin.prevout_index())?;
        data.write_u8(0x19)?;
        data.write_all(tin.script_pubkey())?;
        data.write_u64::<LE>(*tin.value())?;
        data.write_u32::<LE>(tin.sequence().unwrap_or(0xFFFFFFFFu32))?;
        assert_eq!(data.len(), 74);
        buffers.push(data);
    }
    /* hashes
       header:
           version, group, consensus, locktime, expiry: 5*4 = 20
       transparent:
           prevout, sequence, output: 3*32 = 96
       sapling:
           spends, outputs, net: 2*32 + 8 = 72
       orchard: 32
       = 220
    */
    let mut sighashes = vec![];
    let header = pczt.global();
    let expiration = header.expiry_height();
    let version = header.tx_version() | 0x80000000;
    let version_group = header.version_group_id();
    let branch = header.consensus_branch_id();
    sighashes.write_u32::<LE>(version)?;
    sighashes.write_u32::<LE>(*version_group)?;
    sighashes.write_u32::<LE>(*branch)?;
    sighashes.write_u32::<LE>(0)?;
    sighashes.write_u32::<LE>(*expiration)?;

    println!("H: {}", hex::encode(header_hasher(&pczt)?));
    println!("T: {}", hex::encode(transparent_hasher(&pczt)?));
    println!("S: {}", hex::encode(sapling_hasher(&pczt)?));
    println!("O: {}", hex::encode(orchard_hasher(&pczt)?));
    println!("Sig: {}", hex::encode(sig_hasher(&pczt)?));

    sighashes.write_all(&prevout_hasher(&pczt)?)?;
    sighashes.write_all(&sequence_hasher(&pczt)?)?;
    sighashes.write_all(&output_hasher(&pczt)?)?;

    sighashes.write_all(&spend_hasher(&pczt)?)?;
    sighashes.write_all(&zoutput_hasher(&pczt)?)?;
    sighashes.write_i64::<LE>(0)?;

    sighashes.write_all(&orchard_hasher(&pczt)?)?;
    buffers.push(sighashes);

    let check_sign = APDUCommand {
        cla: 0x85,
        ins: 0xA3,
        p1: 0,
        p2: 5,
        data: vec![],
    };

    if stage == 3 {
        let res = ledger.long_execute(&check_sign, &buffers).await?;
        assert_eq!(res.retcode, 0x9000);
        println!(">> {}", hex::encode(&res.data));
    }

    let mut signatures = vec![];
    for _ in tbundle.inputs() {
        let get_signature = APDUCommand {
            cla: 0x85,
            ins: 0xA5,
            p1: 0,
            p2: 0,
            data: vec![],
        };
        if stage == 3 {
            let res = ledger.long_execute(&get_signature, &[vec![]]).await?;
            assert_eq!(res.retcode, 0x9000);
            let signature = res.data[..64].to_vec();
            let signature = Signature::from_compact(&signature).anyhow()?;
            signatures.push(signature);
        }
    }

    let sig_hex = "b154b87733d9040a995880a54e3575b0169920775c080eb71f4c2b9143ca4c454085a5665f997cc8d2652560ee9f775a705500a5236c25f989c70d213ed6b7de";
    if stage == 3 {
        let sig = hex::encode(signatures[0].serialize_compact());
        assert_eq!(sig, sig_hex);
    }

    if stage == 4 {
        let signature = Signature::from_compact(&hex::decode(sig_hex).unwrap()).anyhow()?;
        let signer = Signer::new(pczt.clone());
        let signer = signer
            .sign_transparent_with(|_pczt, tbundle, _| {
                tbundle.inputs_mut()[0].apply_signature(&pk, &signature);
                Ok::<_, zcash_transparent::pczt::ParseError>(())
            })
            .unwrap();
        let pczt = signer.finish();
        let pczt = SpendFinalizer::new(pczt).finalize_spends().unwrap();

        let tx_extractor = TransactionExtractor::new(pczt);
        let tx = tx_extractor.extract().unwrap();
        println!("{}", tx.txid());
        let mut tx_bytes = vec![];
        tx.write(&mut tx_bytes).unwrap();
        println!("{}", hex::encode(&tx_bytes));
    }
    Ok(())
}

#[tokio::test]
pub async fn sign_tx() -> LedgerResult<()> {
    let file = File::open("ledger.bin")?;
    let package = bincode::decode_from_reader::<PcztPackage, _, _>(
        BufReader::new(file),
        bincode::config::legacy(),
    )
    .anyhow()?;
    let pczt = Pczt::parse(&package.pczt).unwrap();

    APDUCommand {
        cla: 0x85,
        ins: 0xA0,
        p1: 0,
        p2: 0,
        data: vec![],
    };

    let mut data = vec![];
    data.write_u8(pczt.transparent().inputs().len() as u8)?;
    data.write_u8(pczt.transparent().outputs().len() as u8)?;
    data.write_u8(pczt.sapling().spends().len() as u8)?;
    data.write_u8(pczt.sapling().outputs().len() as u8)?;

    assert!(pczt.sapling().spends().is_empty());
    assert!(pczt.sapling().outputs().is_empty());

    Ok(())
}

pub fn sapling_hasher(_pczt: &Pczt) -> LedgerResult<[u8; 32]> {
    let hasher = create_hasher(b"ZTxIdSaplingHash");
    Ok(hasher.finalize().as_bytes().try_into().unwrap())
}

pub fn sig_hasher(pczt: &Pczt) -> LedgerResult<[u8; 32]> {
    let mut perso = b"ZcashTxHash_0000".to_vec();
    perso[12..].copy_from_slice(&pczt.global().consensus_branch_id().to_le_bytes());
    let mut hasher = create_hasher(&perso);
    hasher.update(&header_hasher(pczt)?);
    hasher.update(&transparent_hasher(pczt)?);
    hasher.update(&sapling_hasher(pczt)?);
    hasher.update(&orchard_hasher(pczt)?);
    Ok(hasher.finalize().as_bytes().try_into().unwrap())
}
