use std::io::Write;

use anyhow::Result;
use blake2b_simd::{Params, State};
use byteorder::WriteBytesExt;
use pczt::Pczt;
use zcash_primitives::legacy::Script;

pub fn create_hasher(perso: &[u8]) -> State {
    Params::new().hash_length(32).personal(perso).to_state()
}

pub fn header_hasher(pczt: &Pczt) -> Result<[u8; 32]> {
    let mut hasher = create_hasher(b"ZTxIdHeadersHash");
    let header = pczt.global();
    let mut buffer = vec![];
    let version = header.tx_version() | 0x80000000; // overwinter flag
    buffer.write_all(&version.to_le_bytes())?;
    buffer.write_all(&header.version_group_id().to_le_bytes())?;
    buffer.write_all(&header.consensus_branch_id().to_le_bytes())?;
    buffer.write_all(&0u32.to_le_bytes())?;
    buffer.write_all(&header.expiry_height().to_le_bytes())?;
    println!("HEADER: {}", hex::encode(&buffer));
    hasher.update(&buffer);
    Ok(hasher.finalize().as_bytes().try_into().unwrap())
}

pub fn prevout_hasher(pczt: &Pczt) -> Result<[u8; 32]> {
    let mut hasher = create_hasher(b"ZTxIdPrevoutHash");
    for tin in pczt.transparent().inputs() {
        hasher.update(tin.prevout_txid());
        hasher.update(&tin.prevout_index().to_le_bytes());
    }
    Ok(hasher.finalize().as_bytes().try_into().unwrap())
}

pub fn amount_hasher(pczt: &Pczt) -> Result<[u8; 32]> {
    let mut hasher = create_hasher(b"ZTxTrAmountsHash");
    for tin in pczt.transparent().inputs() {
        hasher.update(&tin.value().to_le_bytes());
    }
    Ok(hasher.finalize().as_bytes().try_into().unwrap())
}

pub fn script_hasher(pczt: &Pczt) -> Result<[u8; 32]> {
    let mut hasher = create_hasher(b"ZTxTrScriptsHash");
    for tin in pczt.transparent().inputs() {
        let script = Script(tin.script_pubkey().to_vec());
        script.write(&mut hasher)?;
    }
    Ok(hasher.finalize().as_bytes().try_into().unwrap())
}

pub fn sequence_hasher(pczt: &Pczt) -> Result<[u8; 32]> {
    let mut hasher = create_hasher(b"ZTxIdSequencHash");
    for tin in pczt.transparent().inputs() {
        let sequence = tin.sequence().unwrap_or(0xFFFFFFFFu32);
        hasher.update(&sequence.to_le_bytes());
    }
    Ok(hasher.finalize().as_bytes().try_into().unwrap())
}

pub fn output_hasher(pczt: &Pczt) -> Result<[u8; 32]> {
    let mut hasher = create_hasher(b"ZTxIdOutputsHash");
    for tout in pczt.transparent().outputs() {
        hasher.update(&tout.value().to_le_bytes());
        Script(tout.script_pubkey().to_vec()).write(&mut hasher)?;
    }
    Ok(hasher.finalize().as_bytes().try_into().unwrap())
}

pub fn txin_hasher(pczt: &Pczt, index: usize) -> Result<[u8; 32]> {
    let mut hasher = create_hasher(b"Zcash___TxInHash");
    let tin = &pczt.transparent().inputs()[index];
    hasher.update(tin.prevout_txid());
    hasher.update(&tin.prevout_index().to_le_bytes());
    hasher.update(&tin.value().to_le_bytes());
    Script(tin.script_pubkey().to_vec()).write(&mut hasher)?;
    hasher.update(&tin.sequence().unwrap_or(0xFFFFFFFF).to_le_bytes());
    Ok(hasher.finalize().as_bytes().try_into().unwrap())
}

pub fn transparent_hasher(pczt: &Pczt) -> Result<[u8; 32]> {
    let mut hasher = create_hasher(b"ZTxIdTranspaHash");
    let mut buffer = vec![];
    buffer.write_u8(1u8)?;
    buffer.write_all(&prevout_hasher(pczt)?)?;
    buffer.write_all(&amount_hasher(pczt)?)?;
    buffer.write_all(&script_hasher(pczt)?)?;
    buffer.write_all(&sequence_hasher(pczt)?)?;
    buffer.write_all(&output_hasher(pczt)?)?;
    buffer.write_all(&txin_hasher(pczt, 0)?)?;
    hasher.update(&buffer);
    Ok(hasher.finalize().as_bytes().try_into().unwrap())
}

pub fn sp_compact_hasher(pczt: &Pczt) -> Result<[u8; 32]> {
    let mut hasher = create_hasher(b"ZTxIdSSpendCHash");
    for sin in pczt.sapling().spends() {
        hasher.update(sin.nullifier());
    }
    Ok(hasher.finalize().as_bytes().try_into().unwrap())
}

pub fn sp_noncompact_hasher(pczt: &Pczt) -> Result<[u8; 32]> {
    let mut hasher = create_hasher(b"ZTxIdSSpendNHash");
    for sin in pczt.sapling().spends() {
        hasher.update(sin.cv());
        hasher.update(pczt.sapling().anchor());
        hasher.update(sin.rk());
    }
    Ok(hasher.finalize().as_bytes().try_into().unwrap())
}

pub fn spend_hasher(pczt: &Pczt) -> Result<[u8; 32]> {
    let mut hasher = create_hasher(b"ZTxIdSSpendsHash");
    let mut buffer = vec![];
    buffer.write_all(&sp_compact_hasher(pczt)?)?;
    buffer.write_all(&sp_noncompact_hasher(pczt)?)?;
    hasher.update(&buffer);
    Ok(hasher.finalize().as_bytes().try_into().unwrap())
}

pub fn output_compact_hasher(pczt: &Pczt) -> Result<[u8; 32]> {
    let mut hasher = create_hasher(b"ZTxIdSOutC__Hash");
    for sout in pczt.sapling().outputs() {
        hasher.update(sout.cmu());
        hasher.update(sout.ephemeral_key());
        hasher.update(&sout.enc_ciphertext()[0..52]);
    }
    Ok(hasher.finalize().as_bytes().try_into().unwrap())
}

pub fn output_memo_hasher(pczt: &Pczt) -> Result<[u8; 32]> {
    let mut hasher = create_hasher(b"ZTxIdSOutM__Hash");
    for sout in pczt.sapling().outputs() {
        hasher.update(&sout.enc_ciphertext()[52..564]);
    }
    Ok(hasher.finalize().as_bytes().try_into().unwrap())
}

pub fn output_noncompact_hasher(pczt: &Pczt) -> Result<[u8; 32]> {
    let mut hasher = create_hasher(b"ZTxIdSOutN__Hash");
    for sout in pczt.sapling().outputs() {
        hasher.update(sout.cv());
        hasher.update(&sout.enc_ciphertext()[564..]);
        hasher.update(sout.out_ciphertext());
    }
    Ok(hasher.finalize().as_bytes().try_into().unwrap())
}

pub fn zoutput_hasher(pczt: &Pczt) -> Result<[u8; 32]> {
    let mut hasher = create_hasher(b"ZTxIdSOutputHash");
    let mut buffer = vec![];
    buffer.write_all(&output_compact_hasher(pczt)?)?;
    buffer.write_all(&output_memo_hasher(pczt)?)?;
    buffer.write_all(&output_noncompact_hasher(pczt)?)?;
    hasher.update(&buffer);
    Ok(hasher.finalize().as_bytes().try_into().unwrap())
}

pub fn sapling_hasher(_pczt: &Pczt) -> Result<[u8; 32]> {
    let hasher = create_hasher(b"ZTxIdSaplingHash");
    Ok(hasher.finalize().as_bytes().try_into().unwrap())
}

pub fn orchard_hasher(_pczt: &Pczt) -> Result<[u8; 32]> {
    let hasher = create_hasher(b"ZTxIdOrchardHash");
    Ok(hasher.finalize().as_bytes().try_into().unwrap())
}

pub fn sig_hasher(pczt: &Pczt) -> Result<[u8; 32]> {
    let mut perso = b"ZcashTxHash_0000".to_vec();
    perso[12..].copy_from_slice(&pczt.global().consensus_branch_id().to_le_bytes());
    let mut hasher = create_hasher(&perso);
    hasher.update(&header_hasher(pczt)?);
    hasher.update(&transparent_hasher(pczt)?);
    hasher.update(&sapling_hasher(pczt)?);
    hasher.update(&orchard_hasher(pczt)?);
    Ok(hasher.finalize().as_bytes().try_into().unwrap())
}
