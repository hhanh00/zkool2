// Official Ledger app (LedgerHQ/app-zcash) signing over the PCZT APDU protocol.
//
// Wire contract: docs/PCZT_APDU.md in the app-zcash repository (CLA 0xE0).
// V6 (Ironwood) transactions only: the PCZT fields are framed by hand into the
// app's compact APDU subset, the device reviews and returns spend authorizing /
// transparent signatures, and proofs + binding signature are computed host-side.

use std::collections::HashMap;
use std::io::Write as _;

use anyhow::{anyhow, Result};
use byteorder::{WriteBytesExt, BE, LE};
use ff::PrimeField as _;
use orchard::pczt::Action;
use orchard::primitives::redpallas;
use pczt::roles::{prover::Prover, signer::Signer, spend_finalizer::SpendFinalizer};
use sqlx::{Row, SqliteConnection};
use zcash_keys::encoding::AddressCodec as _;
use zcash_note_encryption::Domain;
use zcash_protocol::consensus::NetworkConstants as _;
use zcash_script::script::Evaluable as _;
use zcash_transparent::address::TransparentAddress;

use crate::{
    account::get_orchard_vk,
    api::{
        coin::Network,
        pay::{PcztPackage, SigningEvent},
    },
    db::{get_account_aindex, get_account_dindex},
    ledger::{
        transport::{APDUCommand, Device},
        LedgerError,
    },
    pay::plan::{get_orchard_pk, IRONWOOD_PK},
    Sink,
};

const CLA: u8 = 0xE0;
const INS_PCZT_HEADER: u8 = 0x52;
const INS_PCZT_TRANSPARENT_INPUT: u8 = 0x53;
const INS_PCZT_TRANSPARENT_OUTPUT: u8 = 0x54;
const INS_PCZT_SIGN_TRANSPARENT: u8 = 0x55;
const INS_PCZT_ORCHARD_ACTION: u8 = 0x56;
const INS_PCZT_SIGN_ORCHARD: u8 = 0x57;
const INS_PCZT_IRONWOOD_ACTION: u8 = 0x58;
const INS_PCZT_SIGN_IRONWOOD: u8 = 0x59;

const P1_FIRST: u8 = 0x00;
const P1_NEXT: u8 = 0x80;
const P1_LAST: u8 = 0x01;
const P2_CONTINUE: u8 = 0x00;
const P2_FINISHED: u8 = 0x01;

const SW_OK: u16 = 0x9000;
const SW_DENY: u16 = 0x6985;

const HARDENED: u32 = 0x8000_0000;
const SIGHASH_ALL: u8 = 0x01;
const PCZT_VERSION_V6: u32 = 2;
const NOTE_VERSION_IRONWOOD: u8 = 0x03;
// The device displays at most 4 shielded outputs across both pools.
const MAX_DISPLAYED_SHIELDED_OUTPUTS: usize = 4;

fn write_compact_size(data: &mut Vec<u8>, n: usize) -> Result<()> {
    if n < 253 {
        data.write_u8(n as u8)?;
    } else if n <= 0xFFFF {
        data.write_u8(0xFD)?;
        data.write_u16::<LE>(n as u16)?;
    } else if n <= 0xFFFF_FFFF {
        data.write_u8(0xFE)?;
        data.write_u32::<LE>(n as u32)?;
    } else {
        data.write_u8(0xFF)?;
        data.write_u64::<LE>(n as u64)?;
    }
    Ok(())
}

fn write_optional_u32(data: &mut Vec<u8>, value: Option<u32>) -> Result<()> {
    match value {
        None => data.write_u8(0x00)?,
        Some(v) => {
            data.write_u8(0x01)?;
            data.write_u32::<LE>(v)?;
        }
    }
    Ok(())
}

fn write_bip32_path(data: &mut Vec<u8>, path: &[u32]) -> Result<()> {
    data.write_u8(path.len() as u8)?;
    for component in path {
        data.write_u32::<BE>(*component)?;
    }
    Ok(())
}

async fn send_command<D: Device>(
    ledger: &D,
    ins: u8,
    packets: Vec<Vec<u8>>,
    finished: bool,
) -> Result<()> {
    let n = packets.len();
    for (i, data) in packets.into_iter().enumerate() {
        if data.len() > 255 {
            anyhow::bail!("APDU packet too long for instruction {ins:#x}");
        }
        let p1 = if i == 0 {
            P1_FIRST
        } else if i == n - 1 {
            P1_LAST
        } else {
            P1_NEXT
        };
        let p2 = if finished && i == n - 1 {
            P2_FINISHED
        } else {
            P2_CONTINUE
        };
        let res = ledger
            .execute(APDUCommand {
                cla: CLA,
                ins,
                p1,
                p2,
                data,
            })
            .await?;
        if res.retcode == SW_DENY {
            anyhow::bail!("the transaction was refused on the Ledger device");
        }
        if res.retcode != SW_OK {
            return Err(LedgerError::Execute(res.retcode, ins).into());
        }
    }
    Ok(())
}

async fn sign_one<D: Device>(ledger: &D, ins: u8, index: u32) -> Result<[u8; 64]> {
    let res = ledger
        .execute(APDUCommand {
            cla: CLA,
            ins,
            p1: 0,
            p2: index as u8,
            data: vec![],
        })
        .await?;
    if res.retcode == SW_DENY {
        anyhow::bail!("the transaction was refused on the Ledger device");
    }
    if res.retcode != SW_OK {
        return Err(LedgerError::Execute(res.retcode, ins).into());
    }
    if res.data.len() != 64 {
        anyhow::bail!("unexpected signature length for instruction {ins:#x}");
    }
    Ok(res.data[..64].try_into().unwrap())
}

// The transparent signing command answers with a DER signature followed by the
// sighash type byte.
async fn sign_transparent_input<D: Device>(
    ledger: &D,
    index: u32,
) -> Result<secp256k1::ecdsa::Signature> {
    let res = ledger
        .execute(APDUCommand {
            cla: CLA,
            ins: INS_PCZT_SIGN_TRANSPARENT,
            p1: 0,
            p2: index as u8,
            data: vec![],
        })
        .await?;
    if res.retcode == SW_DENY {
        anyhow::bail!("the transaction was refused on the Ledger device");
    }
    if res.retcode != SW_OK {
        return Err(LedgerError::Execute(res.retcode, INS_PCZT_SIGN_TRANSPARENT).into());
    }
    let (sighash_type, der) = res
        .data
        .split_last()
        .ok_or_else(|| anyhow!("empty transparent signature response"))?;
    if *sighash_type != SIGHASH_ALL {
        anyhow::bail!("unexpected transparent sighash type {sighash_type:#x}");
    }
    // The app folds the y-parity of the key into the LSB of the DER SEQUENCE
    // tag byte (sig[0] |= 0x01). The parity is irrelevant here — the signature
    // verifies against the pubkey from the PCZT's hash160 preimage — so clear
    // it before parsing.
    let mut der = der.to_vec();
    der[0] &= !0x01;
    secp256k1::ecdsa::Signature::from_der(&der).map_err(|e| {
        anyhow!(
            "invalid DER transparent signature: {e} ({})",
            hex::encode(&der)
        )
    })
}

// Per-action spend fields: cv_net, nullifier, rk, recipient, value, rho,
// rseed, alpha — one packet.
fn frame_spend_small<D: Domain>(action: &Action<D>) -> Result<Vec<u8>> {
    let spend = action.spend();
    let mut data = vec![];
    data.write_all(&action.cv_net().to_bytes())?;
    data.write_all(&spend.nullifier().to_bytes())?;
    data.write_all(&<[u8; 32]>::from(spend.rk()))?;
    let recipient = spend
        .recipient()
        .as_ref()
        .ok_or_else(|| anyhow!("orchard spend has no recipient"))?;
    data.write_all(&recipient.to_raw_address_bytes())?;
    data.write_u64::<LE>(spend.value().ok_or_else(|| anyhow!("orchard spend has no value"))?.inner())?;
    data.write_all(&spend.rho().ok_or_else(|| anyhow!("orchard spend has no rho"))?.to_bytes())?;
    data.write_all(spend.rseed().ok_or_else(|| anyhow!("orchard spend has no rseed"))?.as_bytes())?;
    data.write_all(
        &spend
            .alpha()
            .ok_or_else(|| anyhow!("orchard spend has no alpha"))?
            .to_repr(),
    )?;
    Ok(data)
}

// zip32 derivation packet: seed fingerprint + 32'/coin'/account'. The device
// derives the spending key from this path; the seed fingerprint is not
// validated by the app.
fn frame_zip32(coin_type: u32, aindex: u32) -> Result<Vec<u8>> {
    let mut data = vec![0u8; 32];
    write_bip32_path(&mut data, &[32 | HARDENED, coin_type | HARDENED, aindex | HARDENED])?;
    Ok(data)
}

// Output small fields: cmx + ephemeral key — one packet.
fn frame_output_small<D: Domain>(action: &Action<D>) -> Result<Vec<u8>> {
    let output = action.output();
    let mut data = vec![];
    data.write_all(&output.cmx().to_bytes())?;
    data.write_all(&output.encrypted_note().epk_bytes)?;
    Ok(data)
}

// Large Vec<u8> fields are sent as their own APDU packet sequence: the first
// packet carries the CompactSize byte length followed by as many field bytes
// as fit, continuation packets carry field bytes only (docs/PCZT_APDU.md).
fn frame_large_field(packets: &mut Vec<Vec<u8>>, field: &[u8]) -> Result<()> {
    let mut first = vec![];
    write_compact_size(&mut first, field.len())?;
    let take = (255 - first.len()).min(field.len());
    first.extend_from_slice(&field[..take]);
    packets.push(first);
    let mut rest = &field[take..];
    while !rest.is_empty() {
        let take = rest.len().min(255);
        packets.push(rest[..take].to_vec());
        rest = &rest[take..];
    }
    Ok(())
}

fn frame_enc_ciphertext<D: Domain>(packets: &mut Vec<Vec<u8>>, action: &Action<D>) -> Result<()> {
    let enc: &[u8] = action.output().encrypted_note().enc_ciphertext.as_ref();
    frame_large_field(packets, enc)
}

fn frame_out_ciphertext<D: Domain>(packets: &mut Vec<Vec<u8>>, action: &Action<D>) -> Result<()> {
    let out: &[u8] = &action.output().encrypted_note().out_ciphertext;
    frame_large_field(packets, out)
}

// Output metadata: recipient, value, rseed, rcv (+ notePlaintextVersion 0x03
// for ironwood, making the 116-byte form).
fn frame_output_metadata<D: Domain>(action: &Action<D>, ironwood: bool) -> Result<Vec<u8>> {
    let output = action.output();
    let mut data = vec![];
    let recipient = output
        .recipient()
        .as_ref()
        .ok_or_else(|| anyhow!("orchard output has no recipient"))?;
    data.write_all(&recipient.to_raw_address_bytes())?;
    data.write_u64::<LE>(
        output
            .value()
            .ok_or_else(|| anyhow!("orchard output has no value"))?
            .inner(),
    )?;
    data.write_all(output.rseed().ok_or_else(|| anyhow!("orchard output has no rseed"))?.as_bytes())?;
    data.write_all(
        &action
            .rcv()
            .as_ref()
            .ok_or_else(|| anyhow!("orchard action has no rcv"))?
            .to_bytes(),
    )?;
    if ironwood {
        data.write_u8(NOTE_VERSION_IRONWOOD)?;
    }
    Ok(data)
}

// Bundle trailer: flags, |value_sum|, negative flag, anchor — one packet.
fn frame_trailer<D: Domain>(bundle: &orchard::pczt::Bundle<D>) -> Result<Vec<u8>> {
    let (magnitude, sign) = bundle.value_sum().magnitude_sign();
    let mut data = vec![];
    data.write_u8(bundle.flag_byte())?;
    data.write_u64::<LE>(magnitude)?;
    data.write_u8(matches!(sign, orchard::value::Sign::Negative) as u8)?;
    data.write_all(&(*bundle.anchor()).to_bytes())?;
    Ok(data)
}

// ── Shielded bundle framing ───────────────────────────────────────────────

fn frame_shielded_bundle<D: Domain>(
    bundle: &orchard::pczt::Bundle<D>,
    coin_type: u32,
    aindex: u32,
    internal_change: Option<&[u8; 43]>,
    ironwood: bool,
) -> Result<(Vec<Vec<u8>>, usize)> {
    let mut displayed = 0usize;
    let mut packets = vec![];

    let mut count = vec![];
    write_compact_size(&mut count, bundle.actions().len())?;
    packets.push(count);

    for action in bundle.actions() {
        packets.push(frame_spend_small(action)?);
        packets.push(frame_zip32(coin_type, aindex)?);
        packets.push(frame_output_small(action)?);
        frame_enc_ciphertext(&mut packets, action)?;
        frame_out_ciphertext(&mut packets, action)?;
        packets.push(frame_output_metadata(action, ironwood)?);
        if let Some(recipient) = action.output().recipient().as_ref() {
            let is_change = internal_change.is_some_and(|change| change == &recipient.to_raw_address_bytes());
            if !is_change && action.output().value().is_some_and(|v| v.inner() > 0) {
                displayed += 1;
            }
        }
    }

    if !bundle.actions().is_empty() {
        packets.push(frame_trailer(bundle)?);
    }

    Ok((packets, displayed))
}

pub async fn sign_transaction<D: Device + Sync, S>(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
    package: &PcztPackage,
    sink: Option<&S>,
    ledger: &D,
) -> Result<PcztPackage>
where
    S: Sink<SigningEvent> + Sync,
{
    use pczt::Pczt;
    use pczt::roles::updater::Updater;

    let progress = |msg: String| async move {
        if let Some(sink) = sink {
            sink.send(SigningEvent::Progress(msg)).await;
        }
    };

    progress("Preparing transaction".to_string()).await;

    if package.is_issuance {
        anyhow::bail!("ZSA issuance is not supported on Official Ledger accounts");
    }

    let pczt = Pczt::parse(&package.pczt)
        .map_err(|error| anyhow!("failed to parse PCZT: {error:?}"))?;

    if !pczt.sapling().spends().is_empty() || !pczt.sapling().outputs().is_empty() {
        anyhow::bail!("Official Ledger accounts cannot spend Sapling notes");
    }
    if *pczt.global().tx_version() != 6 {
        anyhow::bail!("only v6 transactions are supported on Official Ledger accounts");
    }

    let coin_type = network.coin_type();
    let aindex = get_account_aindex(connection, account).await?;
    let dindex = get_account_dindex(connection, account).await?;

    // Compressed pubkeys by address, for the transparent input derivation packets.
    let taddrs: Vec<(String, Vec<u8>, u32, u32)> = sqlx::query(
        "SELECT address, pk, scope, dindex FROM transparent_address_accounts WHERE account = ?",
    )
    .bind(account)
    .map(|row: sqlx::sqlite::SqliteRow| {
        (
            row.get::<String, _>(0),
            row.get::<Vec<u8>, _>(1),
            row.get::<u32, _>(2),
            row.get::<u32, _>(3),
        )
    })
    .fetch_all(&mut *connection)
    .await?;

    let pks: HashMap<String, Vec<u8>> = taddrs
        .iter()
        .map(|(address, pk, _, _)| (address.clone(), pk.clone()))
        .collect();
    let mut change_scripts: HashMap<Vec<u8>, (Vec<u8>, u32)> = HashMap::new();
    for (address, pk, scope, dindex) in &taddrs {
        if *scope == 1 {
            if let Ok(taddr) = TransparentAddress::decode(network, address) {
                change_scripts.insert(taddr.script().to_bytes(), (pk.clone(), *dindex));
            }
        }
    }

    // The internal change address is hidden from the device review; every other
    // positive shielded output counts against the device display budget.
    let internal_change = get_orchard_vk(connection, account)
        .await?
        .map(|fvk| {
            fvk.address_at(dindex, orchard::keys::Scope::Internal)
                .to_raw_address_bytes()
        });

    // ── Transparent ───────────────────────────────────────────────────────
    let mut input_packets: Vec<Vec<u8>> = vec![];
    let mut output_packets: Vec<Vec<u8>> = vec![];
    {
        let t = pczt.transparent();
        let mut count = vec![];
        write_compact_size(&mut count, t.inputs().len())?;
        input_packets.push(count);
        for input in t.inputs() {
            let mut small = vec![];
            small.write_all(input.prevout_txid())?;
            small.write_u32::<LE>(*input.prevout_index())?;
            write_optional_u32(&mut small, *input.sequence())?;
            small.write_u64::<LE>(*input.value())?;
            input_packets.push(small);

            let script = input.script_pubkey();
            let mut script_packet = vec![];
            write_compact_size(&mut script_packet, script.len())?;
            script_packet.write_all(script)?;
            input_packets.push(script_packet);

            let address = input
                .proprietary()
                .get("address")
                .ok_or_else(|| anyhow!("transparent input has no address"))?;
            let address = String::from_utf8(address.clone())
                .map_err(|_| anyhow!("invalid transparent input address"))?;
            let pk = pks
                .get(&address)
                .ok_or_else(|| anyhow!("no pubkey for transparent input {address}"))?;
            if pk.len() != 33 {
                anyhow::bail!("transparent input pubkey is not compressed");
            }
            let scope = u32::from_le_bytes(
                input
                    .proprietary()
                    .get("scope")
                    .ok_or_else(|| anyhow!("transparent input has no scope"))?
                    .clone()
                    .try_into()
                    .map_err(|_| anyhow!("invalid scope"))?,
            );
            let input_dindex = u32::from_le_bytes(
                input
                    .proprietary()
                    .get("dindex")
                    .ok_or_else(|| anyhow!("transparent input has no dindex"))?
                    .clone()
                    .try_into()
                    .map_err(|_| anyhow!("invalid dindex"))?,
            );
            let mut signing = vec![SIGHASH_ALL];
            write_compact_size(&mut signing, 1)?;
            signing.write_all(pk)?;
            signing.write_all(&[0u8; 32])?;
            write_bip32_path(
                &mut signing,
                &[
                    44 | HARDENED,
                    coin_type | HARDENED,
                    aindex | HARDENED,
                    scope,
                    input_dindex,
                ],
            )?;
            input_packets.push(signing);
        }

        let mut count = vec![];
        write_compact_size(&mut count, t.outputs().len())?;
        output_packets.push(count);
        for output in t.outputs() {
            let mut value = vec![];
            value.write_u64::<LE>(*output.value())?;
            output_packets.push(value);

            let script = output.script_pubkey();
            let mut script_packet = vec![];
            write_compact_size(&mut script_packet, script.len())?;
            script_packet.write_all(script)?;
            output_packets.push(script_packet);

            let mut derivation = vec![0x00];
            if let Some((pk, dindex)) = change_scripts.get(script) {
                derivation.clear();
                write_compact_size(&mut derivation, 1)?;
                derivation.write_all(pk)?;
                derivation.write_all(&[0u8; 32])?;
                write_bip32_path(
                    &mut derivation,
                    &[
                        44 | HARDENED,
                        coin_type | HARDENED,
                        aindex | HARDENED,
                        1,
                        *dindex,
                    ],
                )?;
            }
            output_packets.push(derivation);
        }
    }

    // ── Shielded bundles ──────────────────────────────────────────────────
    let updater = Updater::new(pczt);
    let mut orchard_framed: Result<(Vec<Vec<u8>>, usize)> = Ok((vec![], 0));
    let updater = updater
        .update_orchard_with(|u| {
            orchard_framed =
                frame_shielded_bundle(u.bundle(), coin_type, aindex, internal_change.as_ref(), false);
            Ok(())
        })
        .map_err(|error| anyhow!("failed to read the orchard bundle: {error:?}"))?;
    let (orchard_packets, orchard_displayed) = orchard_framed?;

    let mut ironwood_framed: Result<(Vec<Vec<u8>>, usize)> = Ok((vec![], 0));
    let updater = updater
        .update_ironwood_with(|u| {
            ironwood_framed =
                frame_shielded_bundle(u.bundle(), coin_type, aindex, internal_change.as_ref(), true);
            Ok(())
        })
        .map_err(|error| anyhow!("failed to read the ironwood bundle: {error:?}"))?;
    let (ironwood_packets, ironwood_displayed) = ironwood_framed?;

    let pczt = updater.finish();

    let displayed_shielded_outputs = orchard_displayed + ironwood_displayed;
    if displayed_shielded_outputs > MAX_DISPLAYED_SHIELDED_OUTPUTS {
        anyhow::bail!(
            "this transaction has {displayed_shielded_outputs} shielded outputs to display, \
             but the Ledger can show at most {MAX_DISPLAYED_SHIELDED_OUTPUTS}"
        );
    }

    // ── Send to the device ────────────────────────────────────────────────
    progress("Confirm on your Ledger".to_string()).await;

    let header = {
        let g = pczt.global();
        let mut data = vec![];
        data.write_all(b"PCZT")?;
        data.write_u32::<LE>(PCZT_VERSION_V6)?;
        data.write_u32::<LE>(*g.tx_version())?;
        data.write_u32::<LE>(*g.version_group_id())?;
        data.write_u32::<LE>(*g.consensus_branch_id())?;
        data.write_u8(0x00)?; // fallback_lock_time: none (builder uses 0)
        data.write_u32::<LE>(*g.expiry_height())?;
        data.write_u32::<LE>(coin_type)?;
        data.write_u8(0x00)?; // tx_modifiable: none
        vec![data]
    };

    send_command(ledger, INS_PCZT_HEADER, header, false).await?;
    send_command(ledger, INS_PCZT_TRANSPARENT_INPUT, input_packets, false).await?;
    send_command(ledger, INS_PCZT_TRANSPARENT_OUTPUT, output_packets, false).await?;
    // V6 defers the review to the ironwood command: it is always sent, even
    // with 0 actions, and its last packet carries P2_FINISHED.
    send_command(ledger, INS_PCZT_ORCHARD_ACTION, orchard_packets, false).await?;
    send_command(ledger, INS_PCZT_IRONWOOD_ACTION, ironwood_packets, true).await?;

    // ── Collect signatures ────────────────────────────────────────────────
    progress("Signing on Ledger".to_string()).await;

    let ctin = pczt.transparent().inputs().len();

    let mut tsigs = Vec::with_capacity(ctin);
    for index in 0..ctin {
        progress(format!(
            "Signing transparent input {}/{}",
            index + 1,
            ctin
        )).await;
        tsigs.push(sign_transparent_input(ledger, index as u32).await?);
    }

    let mut orchard_sigs = Vec::with_capacity(package.orchard_indices.len());
    for index in &package.orchard_indices {
        progress("Signing orchard spend".to_string()).await;
        orchard_sigs.push(sign_one(ledger, INS_PCZT_SIGN_ORCHARD, *index as u32).await?);
    }

    let mut ironwood_sigs = Vec::with_capacity(package.ironwood_indices.len());
    for index in &package.ironwood_indices {
        progress("Signing ironwood spend".to_string()).await;
        ironwood_sigs.push(sign_one(ledger, INS_PCZT_SIGN_IRONWOOD, *index as u32).await?);
    }

    // ── Apply signatures, proofs, binding signature ───────────────────────
    progress("Finalizing transaction".to_string()).await;

    let mut signer = Signer::new(pczt).map_err(|error| anyhow!("signer: {error:?}"))?;
    for (index, sig) in tsigs.iter().enumerate() {
        signer
            .append_transparent_signature(index, *sig)
            .map_err(|error| anyhow!("transparent signature {index}: {error:?}"))?;
    }
    for (index, sig) in package.orchard_indices.iter().zip(&orchard_sigs) {
        let sig = redpallas::Signature::<redpallas::SpendAuth>::from(*sig);
        signer
            .apply_orchard_signature(*index, sig)
            .map_err(|error| anyhow!("orchard signature {index}: {error:?}"))?;
    }
    for (index, sig) in package.ironwood_indices.iter().zip(&ironwood_sigs) {
        let sig = redpallas::Signature::<redpallas::SpendAuth>::from(*sig);
        signer
            .apply_ironwood_signature(*index, sig)
            .map_err(|error| anyhow!("ironwood signature {index}: {error:?}"))?;
    }
    let pczt = signer.finish();

    let orchard_pk = get_orchard_pk(*pczt.global().consensus_branch_id())?;
    let pczt = Prover::new(pczt)
        .create_orchard_proof(orchard_pk)
        .map_err(|error| anyhow!("orchard proof: {error:?}"))?
        .create_ironwood_proof(&IRONWOOD_PK)
        .map_err(|error| anyhow!("ironwood proof: {error:?}"))?
        .finish();

    let pczt = SpendFinalizer::new(pczt)
        .finalize_spends()
        .map_err(|error| anyhow!("failed to finalize spends: {error:?}"))?;

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

    Ok(PcztPackage {
        pczt: pczt
            .serialize()
            .map_err(|error| anyhow!("failed to serialize PCZT: {error:?}"))?,
        n_spends: *n_spends,
        sapling_indices: sapling_indices.clone(),
        orchard_indices: orchard_indices.clone(),
        ironwood_indices: ironwood_indices.clone(),
        can_sign: *can_sign,
        can_broadcast: *can_broadcast,
        price: *price,
        category: *category,
        is_issuance: *is_issuance,
    })
}

pub async fn sign_official_transaction<S>(
    network: Network,
    sink: &S,
    connection: &mut SqliteConnection,
    account: u32,
    package: PcztPackage,
) -> Result<PcztPackage>
where
    S: Sink<SigningEvent> + Sync,
{
    let ledger = crate::ledger::transport::connect_ledger().await?;
    sign_transaction(&network, connection, account, &package, Some(sink), &ledger).await
}
