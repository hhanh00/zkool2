use std::io::Write;

use anyhow::Result;
use byteorder::{WriteBytesExt, LE};
use incrementalmerkletree::Position;
use jubjub::Fr;
use pczt::{
    common::LockTimeInput,
    roles::{
        io_finalizer::IoFinalizer, low_level_signer::Signer, prover::Prover,
        spend_finalizer::SpendFinalizer, updater::Updater,
    },
    Pczt,
};
use rand_core::{CryptoRng, OsRng, RngCore};
use redjubjub::{SpendAuth, VerificationKey, VerificationKeyBytes};
use sapling_crypto::{
    keys::FullViewingKey,
    note_encryption::{sapling_note_encryption, SaplingDomain},
    value::{NoteValue, TrapdoorSum, ValueCommitTrapdoor, ValueCommitment},
    Diversifier, MerklePath, Node, Note, PaymentAddress, ProofGenerationKey, Rseed,
};
use sqlx::{pool::PoolConnection, Sqlite, SqliteConnection};
use tracing::info;
use zcash_keys::encoding::AddressCodec;
use zcash_note_encryption::Domain;
use zcash_primitives::legacy::{Script, TransparentAddress};
use zcash_proofs::prover::LocalTxProver;
use zcash_protocol::memo::{Memo, MemoBytes};

use crate::{
    api::pay::{PcztPackage, SigningEvent},
    coin::Network,
    frb_generated::StreamSink,
    ledger::{
        connect_ledger,
        hashers::{
            orchard_hasher, output_hasher, prevout_hasher, sequence_hasher, spend_hasher,
            zoutput_hasher,
        },
        APDUCommand, Device, LedgerError,
    },
    pay::plan::SAPLING_PROVER,
    tiu,
};

#[allow(clippy::too_many_arguments)]
pub async fn sign_transaction<D: Device, R: RngCore + CryptoRng>(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
    package: &PcztPackage,
    prover: &LocalTxProver,
    sink: &StreamSink<SigningEvent>,
    ledger: &D,
    mut rng: R,
) -> Result<PcztPackage> {
    let (xvk,): (Vec<u8>,) = sqlx::query_as("SELECT xvk FROM sapling_accounts WHERE account = ?1")
        .bind(account)
        .fetch_one(&mut *connection)
        .await?;
    let fvk = FullViewingKey::read(&*xvk)?;

    let pczt = Pczt::parse(&package.pczt).expect("cannot parse PCZT");
    let orig_pczt = pczt.clone();

    let global = pczt.global().clone();

    let mut buffers = vec![];
    let mut data = vec![];

    let ctin = pczt.transparent().inputs().len();
    let ctout = pczt.transparent().outputs().len();
    let stin = pczt.sapling().spends().len();
    let stout = pczt.sapling().outputs().len();
    info!("PCZT structure: {ctin}/{ctout} : {stin}/{stout}");

    let _ = sink.add(SigningEvent::Progress("Init Tx".to_string()));
    data.write_u8(ctin as u8)?;
    data.write_u8(ctout as u8)?;
    data.write_u8(stin as u8)?;
    data.write_u8(stout as u8)?;
    buffers.push(data);

    let mut tins = vec![];
    for tin in pczt.transparent().inputs() {
        let pczt_tin = pczt::transparent::Input::from_parts(
            *tin.prevout_txid(),
            *tin.prevout_index(),
            *tin.sequence(),
            tin.required_time_lock_time(),
            tin.required_height_lock_time(),
            None,
            *tin.value(),
            tin.script_pubkey().clone(),
            None,
            1,
        );
        tins.push(pczt_tin);

        let mut nullifier = vec![];
        nullifier.write_all(tin.prevout_txid())?;
        nullifier.write_u32::<LE>(*tin.prevout_index())?;
        let (scope, dindex, address): (u32, u32, String) = sqlx::query_as(
            "SELECT ta.scope, ta.dindex, ta.address
            FROM notes n
            JOIN transparent_address_accounts ta
            ON ta.id_taddress = n.taddress
            WHERE n.account = ?1 AND n.nullifier = ?2",
        )
        .bind(account)
        .bind(&nullifier)
        .fetch_one(&mut *connection)
        .await?;

        let address = TransparentAddress::decode(&network, &address)?;

        let mut data = vec![];
        data.write_u32::<LE>(44 + 0x80000000)?; // derivation path
        data.write_u32::<LE>(133 + 0x80000000)?;
        data.write_u32::<LE>(0x80000000)?;
        data.write_u32::<LE>(scope)?;
        data.write_u32::<LE>(dindex)?;
        address.script().write(&mut data)?;
        data.write_u64::<LE>(*tin.value())?;
        assert_eq!(data.len(), 54);
        buffers.push(data);
    }

    let mut touts = vec![];
    for tout in pczt.transparent().outputs() {
        let pczt_tout = pczt::transparent::Output::from_parts(
            *tout.value(),
            tout.script_pubkey().clone(),
            None,
        );
        touts.push(pczt_tout);

        let mut data = vec![];
        Script(tout.script_pubkey().to_vec()).write(&mut data)?;
        data.write_u64::<LE>(*tout.value())?;
        assert_eq!(data.len(), 34);
        buffers.push(data);
    }
    let tbundle = pczt::transparent::Bundle::from_parts(tins, touts);

    for sp in pczt.sapling().spends().iter() {
        let nullifier = sp.nullifier();
        let (value, diversifier): (u64, Vec<u8>) = sqlx::query_as(
            "SELECT n.value, n.diversifier FROM notes n LEFT JOIN spends s ON n.id_note = s.id_note
        WHERE s.id_note IS NULL
        AND n.account = ?1 AND n.pool = 1
        AND n.nullifier = ?2",
        )
        .bind(account)
        .bind(&nullifier[..])
        .fetch_one(&mut *connection)
        .await?;

        let diversifier = Diversifier(tiu!(diversifier));
        let recipient = fvk.vk.to_payment_address(diversifier).unwrap();
        let mut data = vec![];
        data.write_u32::<LE>(0)?;
        data.write_all(&recipient.to_bytes())?;
        data.write_u64::<LE>(value)?;
        assert_eq!(data.len(), 55);
        buffers.push(data);
    }

    for sout in pczt.sapling().outputs().iter() {
        let recipient = sout.recipient().unwrap();
        let mut data = vec![];
        data.write_all(&recipient)?;
        data.write_u64::<LE>(sout.value().unwrap())?;
        data.write_u8(0xF6)?;
        data.write_u8(0x01)?; // OVK marker
        data.write_all(&fvk.ovk.0)?;
        assert_eq!(data.len(), 85);
        buffers.push(data);
    }

    let _ = sink.add(SigningEvent::Progress("Confirm Tx on Ledger".to_string()));
    let init_tx = APDUCommand {
        cla: 0x85,
        ins: 0xA0,
        p1: 0,
        p2: 5,
        data: vec![],
    };
    let res = ledger.long_execute(&init_tx, &buffers).await?;
    assert_eq!(res.retcode, 0x9000);

    let mut nsk: [u8; 32] = [0; 32];
    let mut value_sum: i128 = 0;
    let mut bsk = TrapdoorSum::zero();
    let mut sins = vec![];
    let mut rseeds = vec![];
    let mut anchor: Option<[u8; 32]> = None;
    let xtract_sp = APDUCommand {
        cla: 0x85,
        ins: 0xA1,
        p1: 0,
        p2: 0,
        data: vec![],
    };
    for sp in pczt.sapling().spends().iter() {
        let _ = sink.add(SigningEvent::Progress(
            "Extracting spend randomness".to_string(),
        ));
        let res = ledger.execute(&xtract_sp).await?;
        assert_eq!(res.retcode, 0x9000);
        let data = &res.data;
        nsk = tiu!(data[32..64]);
        let ak: [u8; 32] = tiu!(data[0..32]);
        let rcv: [u8; 32] = tiu!(data[64..96]);
        let alpha: [u8; 32] = tiu!(data[96..128]);

        let nullifier = sp.nullifier();
        let (value, position, diversifier, rcm): (u64, u32, Vec<u8>, Vec<u8>)
                = sqlx::query_as(
                "SELECT n.value, n.position, n.diversifier, n.rcm FROM notes n LEFT JOIN spends s ON n.id_note = s.id_note
                WHERE s.id_note IS NULL
                AND n.account = ?1 AND n.pool = 1
                AND n.nullifier = ?2")
                .bind(account)
                .bind(&nullifier[..])
                .fetch_one(&mut *connection)
                .await?;
        value_sum += value as i128;
        let rcm: [u8; 32] = tiu!(rcm);
        let rcv: [u8; 32] = tiu!(rcv);
        let diversifier = Diversifier(tiu!(diversifier));
        let recipient = fvk.vk.to_payment_address(diversifier).unwrap();
        let rseed = Rseed::BeforeZip212(Fr::from_bytes(&rcm).unwrap());
        let rcv2 = ValueCommitTrapdoor::from_bytes(rcv).unwrap();
        let note = Note::from_parts(recipient, NoteValue::from_raw(value), rseed);
        let _nf = note.nf(&fvk.vk.nk, position as u64);

        if anchor.is_none() {
            let witness = sp
                .witness()
                .map(|(position, auth_path_bytes)| {
                    let path_elems = auth_path_bytes
                        .into_iter()
                        .map(|hash| Node::from_bytes(hash).unwrap())
                        .collect::<Vec<_>>();

                    MerklePath::from_parts(path_elems, u64::from(position).into()).unwrap()
                })
                .unwrap();
            let root = witness.root(Node::from_cmu(&note.cmu()));
            anchor = Some(root.to_bytes())
        }

        let pk: VerificationKeyBytes<SpendAuth> = ak.into();
        let pk: VerificationKey<SpendAuth> = tiu!(pk);
        let alpha2 = Fr::from_bytes(&alpha).unwrap();
        let rk = pk.randomize(&alpha2);
        let rk: [u8; 32] = rk.into();

        bsk += &rcv2;
        let cv = ValueCommitment::derive(NoteValue::from_raw(value), rcv2);

        let pczt_sin = pczt::sapling::Spend::from_parts(
            Some(recipient.to_bytes()),
            Some(value),
            cv.to_bytes(),
            *nullifier,
            rk,
            None,
            Some(rcv),
            Some(rcm),
            Some(alpha),
            None,
        );
        sins.push(pczt_sin);
        rseeds.push((rcm, position));
    }

    let ovk = fvk.ovk;
    let xtract_out = APDUCommand {
        cla: 0x85,
        ins: 0xA2,
        p1: 0,
        p2: 0,
        data: vec![],
    };
    let mut souts: Vec<pczt::sapling::Output> = vec![];
    for out in pczt.sapling().outputs().iter() {
        let _ = sink.add(SigningEvent::Progress(
            "Extracting output randomness".to_string(),
        ));
        let res = ledger.execute(&xtract_out).await?;
        assert_eq!(res.retcode, 0x9000);
        let data = &res.data;
        let rcv = &data[0..32];
        let rseed = &data[32..64];
        let rcv: [u8; 32] = tiu!(rcv);
        let rseed: [u8; 32] = tiu!(rseed);

        let value = out.value().unwrap_or_default();
        value_sum -= value as i128;
        let value = NoteValue::from_raw(value);
        let rcv2 = ValueCommitTrapdoor::from_bytes(tiu!(rcv)).unwrap();
        bsk -= &rcv2;

        let cv = ValueCommitment::derive(value, rcv2);
        let recipient = out.recipient().unwrap();
        let recipient = PaymentAddress::from_bytes(&recipient).unwrap();
        let note = Note::from_parts(recipient, value, Rseed::AfterZip212(rseed));
        let cmu = note.cmu();
        // TODO: Support memos
        let memo_bytes: MemoBytes = Memo::Empty.into();
        let note_enc =
            sapling_note_encryption(Some(ovk), note.clone(), memo_bytes.into_bytes(), &mut rng);
        let cout = note_enc.encrypt_outgoing_plaintext(&cv, &cmu, &mut rng);
        let epk = note_enc.epk().to_bytes();
        let enc = note_enc.encrypt_note_plaintext();
        let ock = SaplingDomain::derive_ock(&ovk, &cv, &cmu.to_bytes(), &epk);

        let sout = pczt::sapling::Output::from_parts(
            cv.to_bytes(),
            cmu.to_bytes(),
            Some(recipient.to_bytes()),
            Some(value.inner()),
            tiu!(epk.as_ref()),
            enc.to_vec(),
            cout.to_vec(),
            Some(rseed),
            Some(rcv),
            Some(tiu!(ock.as_ref())),
        );
        souts.push(sout);
    }

    info!("Creating new PCZT");
    let sbundle = pczt::sapling::Bundle::from_parts(sins, souts, value_sum, anchor.unwrap(), None);
    let pczt = pczt::Pczt::from_parts(global, tbundle, sbundle, pczt::orchard::Bundle::default());

    let pczt = IoFinalizer::new(pczt).finalize_io().unwrap();

    let pczt = if !pczt.sapling().spends().is_empty() || !pczt.sapling().outputs().is_empty() {
        let updater = Updater::new(pczt);
        let nsk = Fr::from_bytes(&nsk).unwrap();
        let pgk = ProofGenerationKey { ak: fvk.vk.ak, nsk };

        let updater = updater
            .update_sapling_with(|mut u| {
                for bundle_index in package.sapling_indices.iter() {
                    u.update_spend_with(*bundle_index, |mut u| {
                        u.set_proof_generation_key(pgk.clone()).unwrap();
                        let (position, path) = orig_pczt.sapling().spends()[*bundle_index]
                            .witness()
                            .unwrap();
                        // TODO: Improve - this requires a getter for the witness
                        let path = path
                            .into_iter()
                            .map(|n| Node::from_bytes(n).unwrap())
                            .collect::<Vec<_>>();
                        let witness = incrementalmerkletree::MerklePath::from_parts(
                            path,
                            Position::from(position as u64),
                        );
                        u.set_witness(witness.unwrap());
                        Ok(())
                    })
                    .unwrap();
                }
                Ok(())
            })
            .unwrap();
        updater.finish()
    } else {
        pczt
    };

    info!("Adding proofs to PCZT");
    let _ = sink.add(SigningEvent::Progress("Computing ZKPs".to_string()));
    let pczt = Prover::new(pczt)
        .create_sapling_proofs(prover, prover)
        .unwrap()
        .finish();

    let mut buffers = vec![];
    for tin in pczt.transparent().inputs() {
        let mut data = vec![];
        data.write_all(tin.prevout_txid())?;
        data.write_u32::<LE>(*tin.prevout_index())?;
        Script(tin.script_pubkey().to_vec()).write(&mut data)?;
        data.write_u64::<LE>(*tin.value())?;
        data.write_u32::<LE>(tin.sequence().unwrap_or(0xFFFFFFFFu32))?;
        assert_eq!(data.len(), 74);
        buffers.push(data);
    }
    for (rcm, position) in rseeds.iter() {
        let mut data = vec![];
        data.write_all(rcm)?;
        data.write_u64::<LE>(*position as u64)?;
        assert_eq!(data.len(), 40);
        buffers.push(data);
    }
    let anchor = pczt.sapling().anchor();
    for sin in pczt.sapling().spends() {
        let mut data = vec![];
        data.write_all(sin.cv())?;
        data.write_all(anchor)?;
        data.write_all(sin.nullifier())?;
        data.write_all(sin.rk())?;
        data.write_all(&sin.zkproof().unwrap())?;
        assert_eq!(data.len(), 320);
        buffers.push(data);
    }
    for sout in pczt.sapling().outputs() {
        let mut data = vec![];
        data.write_all(sout.cv())?;
        data.write_all(sout.cmu())?;
        data.write_all(sout.ephemeral_key())?;
        data.write_all(sout.enc_ciphertext())?;
        data.write_all(sout.out_ciphertext())?;
        data.write_all(&sout.zkproof().unwrap())?;
        assert_eq!(data.len(), 948);
        buffers.push(data);
    }

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

    sighashes.write_all(&prevout_hasher(&pczt)?)?;
    sighashes.write_all(&sequence_hasher(&pczt)?)?;
    sighashes.write_all(&output_hasher(&pczt)?)?;

    sighashes.write_all(&spend_hasher(&pczt)?)?;
    sighashes.write_all(&zoutput_hasher(&pczt)?)?;
    sighashes.write_i64::<LE>(*pczt.sapling().value_sum() as i64)?;
    sighashes.write_all(&orchard_hasher(&pczt)?)?;

    assert_eq!(sighashes.len(), 220);
    buffers.push(sighashes);

    let _ = sink.add(SigningEvent::Progress(
        "Checking Tx and Signing on Ledger".to_string(),
    ));
    let check_sign = APDUCommand {
        cla: 0x85,
        ins: 0xA3,
        p1: 0,
        p2: 5,
        data: vec![],
    };

    let res = ledger.long_execute(&check_sign, &buffers).await?;
    assert_eq!(res.retcode, 0x9000);

    let mut tsigs = vec![];
    for _ in pczt.transparent().inputs() {
        let _ = sink.add(SigningEvent::Progress(
            "Getting transparent signature".to_string(),
        ));
        let get_tsig = APDUCommand {
            cla: 0x85,
            ins: 0xA5,
            p1: 0,
            p2: 0,
            data: vec![],
        };
        let res = ledger.execute(&get_tsig).await?;
        assert_eq!(res.retcode, 0x9000);
        let signature = res.data[..64].to_vec();
        let signature = secp256k1::ecdsa::Signature::from_compact(&signature)?;
        tsigs.push(signature);
    }

    let mut ssigs = vec![];
    for _ in pczt.sapling().spends() {
        let _ = sink.add(SigningEvent::Progress(
            "Getting shielded signature".to_string(),
        ));
        let get_ssig = APDUCommand {
            cla: 0x85,
            ins: 0xA4,
            p1: 0,
            p2: 0,
            data: vec![],
        };
        let res = ledger.execute(&get_ssig).await?;
        assert_eq!(res.retcode, 0x9000);
        let signature: [u8; 64] = tiu!(res.data[..64].to_vec());
        let signature: redjubjub::Signature<SpendAuth> = tiu!(signature);
        ssigs.push(signature);
    }

    let signer = Signer::new(pczt.clone());
    let signer = signer
        .sign_sapling_with(|_pczt, sbundle, _| {
            for (sp, signature) in sbundle.spends_mut().iter_mut().zip(ssigs.iter()) {
                sp.apply_signature(*signature);
            }
            Ok::<_, sapling_crypto::pczt::ParseError>(())
        })
        .unwrap();
    let pczt = signer.finish();
    let pczt = SpendFinalizer::new(pczt).finalize_spends().unwrap();

    let PcztPackage {
        n_spends,
        sapling_indices,
        orchard_indices,
        can_sign,
        can_broadcast,
        price,
        category,
        ..
    } = package;

    let new_package = PcztPackage {
        pczt: pczt.serialize(),
        n_spends: *n_spends,
        sapling_indices: sapling_indices.clone(),
        orchard_indices: orchard_indices.clone(),
        can_sign: *can_sign,
        can_broadcast: *can_broadcast,
        price: *price,
        category: *category,
    };
    Ok(new_package)
}

pub async fn sign_ledger_transaction(
    network: Network,
    sink: StreamSink<SigningEvent>,
    mut connection: PoolConnection<Sqlite>,
    account: u32,
    package: PcztPackage,
) -> Result<()> {
    tokio::spawn(async move {
        let ledger = connect_ledger().await?;
        let new_package = sign_transaction(
            &network,
            &mut connection,
            account,
            &package,
            &SAPLING_PROVER,
            &sink,
            &ledger,
            OsRng,
        )
        .await?;
        let _ = sink.add(SigningEvent::Result(new_package));
        Ok::<_, LedgerError>(())
    });
    Ok(())
}
