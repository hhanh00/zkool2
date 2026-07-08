use std::io::Write;

use anyhow::Result;
use byteorder::{WriteBytesExt, LE};
use jubjub::Fr;
use pczt::{
    roles::{
        io_finalizer::IoFinalizer, prover::Prover,
        spend_finalizer::SpendFinalizer, updater::Updater,
    },
    Pczt,
};
use rand_core::{CryptoRng, OsRng, RngCore};
use redjubjub::{SpendAuth, VerificationKey, VerificationKeyBytes};
use sapling_crypto::{
    bundle::OutputDescription,
    keys::FullViewingKey,
    note::ExtractedNoteCommitment,
    note_encryption::{sapling_note_encryption, SaplingDomain, Zip212Enforcement},
    value::{NoteValue, ValueCommitTrapdoor, ValueCommitment},
    Diversifier, Note, PaymentAddress, ProofGenerationKey, Rseed,
};
use secp256k1::PublicKey;
use sqlx::{pool::PoolConnection, Sqlite, SqliteConnection};
use tracing::info;
use zcash_keys::encoding::AddressCodec;
use zcash_note_encryption::{try_output_recovery_with_ovk, Domain, EphemeralKeyBytes, OutgoingCipherKey};
use zcash_script::script::Evaluable;
use zcash_transparent::address::TransparentAddress;
use zcash_proofs::prover::LocalTxProver;
use zcash_protocol::{consensus::NetworkConstants, memo::Memo};

use crate::{
    api::pay::{PcztPackage, SigningEvent},
    api::coin::Network,
    db::get_account_aindex,
    ledger::{
        hashers::{
            orchard_hasher, output_hasher, prevout_hasher, sequence_hasher, spend_hasher,
            zoutput_hasher,
        },
        LedgerError, LedgerResult,
    },
    pay::plan::get_sapling_prover,
    tiu, IntoAnyhow,
};

#[cfg(feature = "flutter")]
use crate::{frb_generated::StreamSink, ledger::transport::Device};

#[allow(clippy::too_many_arguments)]
#[cfg(feature = "flutter")]
pub async fn sign_transaction<D: Device + Sync, R: RngCore + CryptoRng>(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
    package: &PcztPackage,
    prover: &LocalTxProver,
    sink: &StreamSink<SigningEvent>,
    ledger: &D,
    mut rng: R,
) -> LedgerResult<()> {
    let s = sink;
    let run = async move {
        use crate::ledger::transport::APDUCommand;

        let coin_type = network.coin_type();
        let aindex = get_account_aindex(&mut *connection, account).await?;
        let (xvk,): (Vec<u8>,) =
            sqlx::query_as("SELECT xvk FROM sapling_accounts WHERE account = ?1")
                .bind(account)
                .fetch_one(&mut *connection)
                .await
                .anyhow()?;
        let fvk = FullViewingKey::read(&*xvk)?;
        let ovk = fvk.ovk;

        let pczt = Pczt::parse(&package.pczt).expect("cannot parse PCZT");

        let mut buffers = vec![];
        let mut data = vec![];

        let ctin = pczt.transparent().inputs().len();
        let ctout = pczt.transparent().outputs().len();
        let stin = pczt.sapling().spends().len();
        let stout = pczt.sapling().outputs().len();
        if !pczt.orchard().actions().is_empty() {
            return Err(LedgerError::HasOrchard);
        }
        info!("PCZT structure: {ctin}/{ctout} : {stin}/{stout}");
        if ctin > 5 || ctout > 5 || stin > 5 || stout > 5 {
            return Err(LedgerError::TooComplex);
        }

        // Signing a tx with the Ledger involves several steps
        // Step 1. Send a InitTx instruction with inputs/outputs
        let _ = sink.add(SigningEvent::Progress("Init Tx".to_string()));
        data.write_u8(ctin as u8)?;
        data.write_u8(ctout as u8)?;
        data.write_u8(stin as u8)?;
        data.write_u8(stout as u8)?;
        buffers.push(data);

        let mut pks = vec![];
        for tin in pczt.transparent().inputs() {
            let mut nullifier = vec![];
            nullifier.write_all(tin.prevout_txid())?;
            nullifier.write_u32::<LE>(*tin.prevout_index())?;
            let (scope, dindex, pk, address): (u32, u32, Vec<u8>, String) = sqlx::query_as(
                "SELECT ta.scope, ta.dindex, ta.pk, ta.address
            FROM notes n
            JOIN transparent_address_accounts ta
            ON ta.id_taddress = n.taddress
            WHERE n.account = ?1 AND n.nullifier = ?2",
            )
            .bind(account)
            .bind(&nullifier)
            .fetch_one(&mut *connection)
            .await
            .anyhow()?;

            let pk = PublicKey::from_slice(&pk).anyhow()?;
            pks.push(pk);
            let address = TransparentAddress::decode(&network, &address).anyhow()?;

            let mut data = vec![];
            data.write_u32::<LE>(44 + 0x80000000)?; // derivation path
            data.write_u32::<LE>(coin_type | 0x80000000)?;
            data.write_u32::<LE>(aindex | 0x80000000)?;
            data.write_u32::<LE>(scope)?;
            data.write_u32::<LE>(dindex)?;
            let address_script = address.script();
            data.write_u8(address_script.byte_len() as u8)?;
            data.write_all(&address_script.to_bytes())?;
            data.write_u64::<LE>(*tin.value())?;
            assert_eq!(data.len(), 54);
            buffers.push(data);
        }

        for tout in pczt.transparent().outputs() {
            let mut data = vec![];
            let output_script = tout.script_pubkey();
            data.write_u8(output_script.len() as u8)?;
            data.write_all(output_script)?;
            data.write_u64::<LE>(*tout.value())?;
            assert_eq!(data.len(), 34);
            buffers.push(data);
        }
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
            .await.anyhow()?;

            let diversifier = Diversifier(tiu!(diversifier));
            let recipient = fvk.vk.to_payment_address(diversifier).unwrap();
            let mut data = vec![];
            data.write_u32::<LE>(aindex)?;
            data.write_all(&recipient.to_bytes())?;
            data.write_u64::<LE>(value)?;
            assert_eq!(data.len(), 55);
            buffers.push(data);
        }

        let mut memos = vec![];
        for sout in pczt.sapling().outputs().iter() {
            let recipient = sout.recipient().unwrap();
            // Decrypt the memo so that we can reencrypt it with the
            // randomness
            let cmu: [u8; 32] = tiu!(*sout.cmu());
            let cv: [u8; 32] = tiu!(*sout.cv());
            let epk: [u8; 32] = tiu!(*sout.ephemeral_key());
            let enc: [u8; 580] = tiu!(sout.enc_ciphertext().clone());
            let cout: [u8; 80] = tiu!(sout.out_ciphertext().clone());
            let cv = ValueCommitment::from_bytes_not_small_order(&cv).unwrap();
            let od = OutputDescription::from_parts(
                cv.clone(),
                ExtractedNoteCommitment::from_bytes(&cmu).unwrap(),
                EphemeralKeyBytes(epk),
                enc,
                cout,
                (),
            );

            let memo = match try_output_recovery_with_ovk(
                &SaplingDomain::new(Zip212Enforcement::On),
                &ovk,
                &od,
                &cv,
                &cout,
            ) {
                Some((_, _, memo)) => memo,
                None => {
                    let memo = Memo::Empty;
                    let memo_bytes = memo.encode();
                    let memo: [u8; 512] = tiu!(*memo_bytes.as_array());
                    memo
                },
            };
            let memo_type = memo[0];
            memos.push(memo);

            let mut data = vec![];
            data.write_all(&recipient)?;
            data.write_u64::<LE>(sout.value().unwrap())?;
            data.write_u8(memo_type)?;
            data.write_u8(0x01)?; // OVK marker
            data.write_all(&fvk.ovk.0)?;
            assert_eq!(data.len(), 85);
            buffers.push(data);
        }

        // This will make the Ledger show "Please Review..."
        info!("Confirm Tx on Ledger");
        let _ = sink.add(SigningEvent::Progress("Confirm Tx on Ledger".to_string()));
        let init_tx = APDUCommand {
            cla: 0x85,
            ins: 0xA0,
            p1: 0,
            p2: 5,
            data: vec![],
        };
        let res = ledger.long_execute(&init_tx, &buffers).await?;
        if res.retcode != 0x9000 {
            return Err(LedgerError::Execute(res.retcode, init_tx.ins));
        }

        // On confirmation, we move on to the next step
        // Step 2. Retrieve the random values for the spends and outputs
        // These values are generated on the Ledger and must be used
        // by the PCZT. We collect the data then apply it in-place via the Updater.
        struct SpendUpdate {
            rcv: [u8; 32],
            alpha: [u8; 32],
            rseed: Rseed,
            value: u64,
            ak: [u8; 32],
        }
        struct OutputUpdate {
            rcv: [u8; 32],
            rseed: [u8; 32],
            value: u64,
            recipient_bytes: [u8; 43],
            epk_bytes: [u8; 32],
            enc: Vec<u8>,
            cout: Vec<u8>,
            ock_bytes: [u8; 32],
        }
        let mut nsk: [u8; 32] = [0; 32];
        let mut spend_updates: Vec<SpendUpdate> = vec![];
        let mut output_updates: Vec<OutputUpdate> = vec![];
        let mut rseeds = vec![];
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
            let res = ledger.execute(xtract_sp.clone()).await?;
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
                .await.anyhow()?;
            let rcm: [u8; 32] = tiu!(rcm);
            let rcv: [u8; 32] = tiu!(rcv);
            let diversifier = Diversifier(tiu!(diversifier));
            let recipient = fvk.vk.to_payment_address(diversifier).unwrap();
            let rseed = Rseed::BeforeZip212(Fr::from_bytes(&rcm).unwrap());
            let note = Note::from_parts(recipient, NoteValue::from_raw(value), rseed);
            let _nf = note.nf(&fvk.vk.nk, position as u64);

            // Store data for in-place modification via Updater
            spend_updates.push(SpendUpdate {
                rcv,
                alpha,
                rseed,
                value,
                ak,
            });

            rseeds.push((rcm, position));
        }

        let xtract_out = APDUCommand {
            cla: 0x85,
            ins: 0xA2,
            p1: 0,
            p2: 0,
            data: vec![],
        };
        for (out, memo) in pczt.sapling().outputs().iter().zip(memos.iter()) {
            let _ = sink.add(SigningEvent::Progress(
                "Extracting output randomness".to_string(),
            ));
            let res = ledger.execute(xtract_out.clone()).await?;
            assert_eq!(res.retcode, 0x9000);
            let data = &res.data;
            let rcv = &data[0..32];
            let rseed = &data[32..64];
            let rcv: [u8; 32] = tiu!(rcv);
            let rseed: [u8; 32] = tiu!(rseed);

            let value = out.value().unwrap_or_default();
            let value_note = NoteValue::from_raw(value);
            let rcv2 = ValueCommitTrapdoor::from_bytes(tiu!(rcv)).unwrap();

            let cv = ValueCommitment::derive(value_note, rcv2);
            let recipient = out.recipient().unwrap();
            let recipient = PaymentAddress::from_bytes(&recipient).unwrap();
            let note = Note::from_parts(recipient, value_note, Rseed::AfterZip212(rseed));
            let cmu = note.cmu();

            let note_enc = sapling_note_encryption(Some(ovk), note.clone(), *memo, &mut rng);
            let cout = note_enc.encrypt_outgoing_plaintext(&cv, &cmu, &mut rng);
            let epk = SaplingDomain::epk_bytes(note_enc.epk());
            let enc = note_enc.encrypt_note_plaintext();
            let ock = SaplingDomain::derive_ock(&ovk, &cv, &cmu.to_bytes(), &epk);

            let recipient_bytes: [u8; 43] = tiu!(recipient.to_bytes());
            output_updates.push(OutputUpdate {
                rcv,
                rseed,
                value,
                recipient_bytes,
                epk_bytes: tiu!(epk.as_ref()),
                enc: enc.0.to_vec(),
                cout: cout.to_vec(),
                ock_bytes: tiu!(ock.as_ref()),
            });
        }

        // Apply all Ledger random values in-place via the Updater
        info!("Applying Ledger randomness to PCZT");
        let pczt = Updater::new(pczt)
            .update_sapling_with(|mut u| {
                for (i, su) in spend_updates.iter().enumerate() {
                    u.update_spend_with(i, |mut su_updater| {
                        let rcv = ValueCommitTrapdoor::from_bytes(su.rcv).unwrap();
                        let alpha = jubjub::Fr::from_bytes(&su.alpha).unwrap();
                        let cv = ValueCommitment::derive(
                            NoteValue::from_raw(su.value),
                            rcv,
                        );
                        let pk: VerificationKeyBytes<SpendAuth> = su.ak.into();
                        let pk: VerificationKey<SpendAuth> = pk.try_into().expect("valid ak from Ledger");
                        let rk = pk.randomize(&alpha);
                        // Re-derive rcv since ValueCommitment::derive consumed it
                        let rcv2 = ValueCommitTrapdoor::from_bytes(su.rcv).unwrap();
                        su_updater.set_cv(cv);
                        su_updater.set_rk(rk);
                        su_updater.set_rcv(rcv2);
                        su_updater.set_alpha(alpha);
                        su_updater.set_rseed(su.rseed);
                        Ok(())
                    }).unwrap();
                }
                for (i, ou) in output_updates.iter().enumerate() {
                    u.update_output_with(i, |mut ou_updater| {
                        let rcv = ValueCommitTrapdoor::from_bytes(ou.rcv).unwrap();
                        let value_note = NoteValue::from_raw(ou.value);
                        let cv = ValueCommitment::derive(value_note, rcv);
                        let rcv2 = ValueCommitTrapdoor::from_bytes(ou.rcv).unwrap();
                        let recipient = PaymentAddress::from_bytes(&ou.recipient_bytes).unwrap();
                        let note = Note::from_parts(
                            recipient,
                            value_note,
                            Rseed::AfterZip212(ou.rseed),
                        );
                        let cmu = note.cmu();
                        let epk = EphemeralKeyBytes(ou.epk_bytes);
                        let ock = OutgoingCipherKey(ou.ock_bytes);
                        ou_updater.set_cv(cv);
                        ou_updater.set_cmu(cmu);
                        ou_updater.set_ephemeral_key(epk);
                        ou_updater.set_enc_ciphertext(tiu!(ou.enc.as_slice()));
                        ou_updater.set_out_ciphertext(tiu!(ou.cout.as_slice()));
                        ou_updater.set_rcv(rcv2);
                        ou_updater.set_rseed(ou.rseed);
                        ou_updater.set_ock(ock);
                        Ok(())
                    }).unwrap();
                }
                Ok(())
            })
            .unwrap()
            .finish();

        let (pczt, _) = IoFinalizer::new(pczt).finalize_io().unwrap();

        let pczt = if !pczt.sapling().spends().is_empty() || !pczt.sapling().outputs().is_empty() {
            let updater = Updater::new(pczt);
            let nsk = Fr::from_bytes(&nsk).unwrap();
            let pgk = ProofGenerationKey { ak: fvk.vk.ak, nsk };

            let updater = updater
                .update_sapling_with(|mut u| {
                    for bundle_index in package.sapling_indices.iter() {
                        // Read witness before the mutable borrow on update_spend_with
                        let witness = u.bundle().spends()[*bundle_index]
                            .witness()
                            .clone()
                            .expect("spend must have a witness");
                        u.update_spend_with(*bundle_index, |mut su| {
                            su.set_proof_generation_key(pgk.clone()).unwrap();
                            su.set_witness(witness);
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

        // The signing steps requires the newly computed spends/outputs
        // and a bunch of hashes as defined in the Tx v5 (ZIP 244)
        let mut buffers = vec![];
        for tin in pczt.transparent().inputs() {
            let mut data = vec![];
            data.write_all(tin.prevout_txid())?;
            data.write_u32::<LE>(*tin.prevout_index())?;
            let input_script = tin.script_pubkey();
            data.write_u8(input_script.len() as u8)?;
            data.write_all(input_script)?;
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
        // Read zkproof from sapling-crypto types (pczt types don't expose it)
        let anchor = *pczt.sapling().anchor();
        // Use update_sapling_with to access sapling-crypto Spend/Output which have full
        // getters including zkproof()
        let mut proof_bufs: Vec<Vec<u8>> = vec![];
        {
            let pczt_for_read = pczt.clone();
            let _ = Updater::new(pczt_for_read)
                .update_sapling_with(|u| {
                    let bundle = u.bundle();
                    for sin in bundle.spends() {
                        let mut data = vec![];
                        data.write_all(&sin.cv().to_bytes()).unwrap();
                        data.write_all(&anchor).unwrap();
                        data.write_all(sin.nullifier().as_ref()).unwrap();
                        let rk_bytes: [u8; 32] = VerificationKeyBytes::from(*sin.rk()).into();
                        data.write_all(&rk_bytes).unwrap();
                        let zkp = sin.zkproof().expect("spend must have zkproof after proving");
                        data.write_all(zkp.as_ref()).unwrap();
                        assert_eq!(data.len(), 320);
                        proof_bufs.push(data);
                    }
                    for sout in bundle.outputs() {
                        let mut data = vec![];
                        data.write_all(&sout.cv().to_bytes()).unwrap();
                        data.write_all(&sout.cmu().to_bytes()).unwrap();
                        data.write_all(sout.ephemeral_key().as_ref()).unwrap();
                        data.write_all(sout.enc_ciphertext()).unwrap();
                        data.write_all(sout.out_ciphertext()).unwrap();
                        let zkp = sout.zkproof().expect("output must have zkproof after proving");
                        data.write_all(zkp.as_ref()).unwrap();
                        assert_eq!(data.len(), 948);
                        proof_bufs.push(data);
                    }
                    Ok(())
                })
                .unwrap();
        }
        buffers.extend(proof_bufs);

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

        // We are now ready to get the signatures for each inputs
        // Starting from the transparent inputs
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
            let res = ledger.execute(get_tsig).await?;
            assert_eq!(res.retcode, 0x9000);
            let signature = res.data[..64].to_vec();
            let signature = secp256k1::ecdsa::Signature::from_compact(&signature).anyhow()?;
            tsigs.push(signature);
        }

        // And then the shielded spends
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
            let res = ledger.execute(get_ssig).await?;
            assert_eq!(res.retcode, 0x9000);
            let signature: [u8; 64] = tiu!(res.data[..64].to_vec());
            let signature: redjubjub::Signature<SpendAuth> = tiu!(signature);
            ssigs.push(signature);
        }

        // We apply these signatures to the PCZT.
        // First populate hash160_preimages so append_transparent_signature can find pubkeys.
        let updater = Updater::new(pczt);
        let updater = updater
            .update_transparent_with(|mut u| {
                for (i, pk) in pks.iter().enumerate() {
                    u.update_input_with(i, |mut u| {
                        u.set_hash160_preimage(pk.serialize().to_vec());
                        Ok(())
                    })?;
                }
                Ok(())
            })
            .unwrap();
        let pczt = updater.finish();

        let mut signer = pczt::roles::signer::Signer::new(pczt).unwrap();
        for (index, signature) in tsigs.iter().enumerate() {
            signer.append_transparent_signature(index, *signature).unwrap();
        }
        for (index, signature) in ssigs.iter().enumerate() {
            signer.apply_sapling_signature(index, *signature).unwrap();
        }
        let pczt = signer.finish();
        // And calculate the final components, i.e the binding signature
        let pczt = SpendFinalizer::new(pczt).finalize_spends().unwrap();

        // Then we rebuild a PCZT (package) identical to the input
        // ones, but with signatures and different random values
        let PcztPackage {
            n_spends,
            sapling_indices,
            orchard_indices,
            ironwood_indices,
            can_sign,
            can_broadcast,
            price,
            category,
            is_issuance,
            ..
        } = package;

        // The new package is ready to be broadcast
        let new_package = PcztPackage {
            pczt: pczt.serialize().unwrap(),
            n_spends: *n_spends,
            sapling_indices: sapling_indices.clone(),
            orchard_indices: orchard_indices.clone(),
            ironwood_indices: ironwood_indices.clone(),
            can_sign: *can_sign,
            can_broadcast: *can_broadcast,
            price: *price,
            category: *category,
            is_issuance: *is_issuance,
        };
        Ok(new_package)
    };
    match run.await {
        Ok(new_package) => {
            let _ = sink.add(SigningEvent::Result(new_package));
        }
        Err(error) => {
            let _ = s.add_error(anyhow::Error::new(error));
        }
    }
    Ok(())
}

#[cfg(feature = "flutter")]
pub async fn sign_ledger_transaction(
    network: Network,
    sink: StreamSink<SigningEvent>,
    mut connection: PoolConnection<Sqlite>,
    account: u32,
    package: PcztPackage,
) -> Result<()> {
    tokio::spawn(async move {
        use crate::ledger::transport::connect_ledger;

        let ledger = connect_ledger().await?;
        let sapling_prover = get_sapling_prover().await?;
        sign_transaction(
            &network,
            &mut connection,
            account,
            &package,
            sapling_prover,
            &sink,
            &ledger,
            OsRng,
        )
        .await?;
        Ok::<_, LedgerError>(())
    });
    Ok(())
}
