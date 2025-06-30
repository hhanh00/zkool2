use crate::{
    lwd::{CompactOrchardAction, CompactSaplingOutput},
    sync::Note,
};

use anyhow::Result;
use blake2b_simd::Params;
use chacha20::{
    cipher::{KeyIvInit, StreamCipher, StreamCipherSeek},
    ChaCha20,
};
use halo2_proofs::pasta::{
    group::{ff::PrimeField as _, Curve, GroupEncoding as _},
    pallas::Point,
    Fq,
};
use orchard::{
    keys::IncomingViewingKey,
    note::{ExtractedNoteCommitment, Nullifier, Rho},
    note_encryption::{CompactAction, OrchardDomain},
};
use sapling_crypto::{
    note_encryption::{plaintext_version_is_valid, SaplingDomain, KDF_SAPLING_PERSONALIZATION},
    SaplingIvk,
};
use zcash_note_encryption::{EphemeralKeyBytes, COMPACT_NOTE_SIZE};
use zcash_primitives::transaction::components::sapling::zip212_enforcement;
use zcash_protocol::consensus::Network;

#[allow(clippy::too_many_arguments)]
pub fn try_sapling_decrypt(
    network: &Network,
    account: u32,
    scope: u8,
    ivk: &SaplingIvk,
    height: u32,
    ivtx: u32,
    vout: u32,
    co: &CompactSaplingOutput,
) -> Result<Option<(sapling_crypto::Note, crate::sync::Note)>> {
    let epkb = &*co.epk;
    let epk = jubjub::AffinePoint::from_bytes(epkb.try_into().unwrap()).unwrap();
    let enc = &co.ciphertext;
    let epk = epk.mul_by_cofactor().to_niels();
    let zip212_enforcement = zip212_enforcement(network, height.into());
    let ka = epk.multiply_bits(&ivk.to_repr()).to_affine();
    let key = Params::new()
        .hash_length(32)
        .personal(KDF_SAPLING_PERSONALIZATION)
        .to_state()
        .update(&ka.to_bytes())
        .update(epkb)
        .finalize();
    let mut plaintext = [0; COMPACT_NOTE_SIZE];
    plaintext.copy_from_slice(enc);
    let mut keystream = ChaCha20::new(key.as_ref().into(), [0u8; 12][..].into());
    keystream.seek(64);
    keystream.apply_keystream(&mut plaintext);
    if (plaintext[0] == 0x01 || plaintext[0] == 0x02)
        && plaintext_version_is_valid(zip212_enforcement, plaintext[0])
    {
        use zcash_note_encryption::Domain;
        let pivk = sapling_crypto::keys::PreparedIncomingViewingKey::new(ivk);
        let d = SaplingDomain::new(zip212_enforcement);
        if let Some((note, recipient)) = d.parse_note_plaintext_without_memo_ivk(&pivk, &plaintext)
        {
            let cmx = note.cmu();
            if cmx.to_bytes() == *co.cmu {
                let value = note.value().inner();
                let dbn = Note {
                    pool: 1,
                    account,
                    scope,
                    height,
                    value,
                    rcm: note.rcm().to_bytes().to_vec(),
                    vout,
                    diversifier: recipient.diversifier().0.to_vec(),
                    ivtx,
                    cmx: cmx.to_bytes().to_vec(),
                    // nf cannot be calculated at this point because we don't have the position
                    ..Note::default()
                };
                return Ok(Some((note, dbn)));
            }
        }
    }
    Ok(None)
}

const KDF_ORCHARD_PERSONALIZATION: &[u8; 16] = b"Zcash_OrchardKDF";

#[allow(clippy::too_many_arguments)]
pub fn try_orchard_decrypt(
    network: &Network,
    account: u32,
    scope: u8,
    ivk: &IncomingViewingKey,
    height: u32,
    ivtx: u32,
    vout: u32,
    ca: &CompactOrchardAction,
) -> Result<Option<(orchard::note::Note, Note)>> {
    let zip212_enforcement = zip212_enforcement(network, height.into());
    let bb = ivk.to_bytes();
    let ivk_fq = Fq::from_repr(bb[32..64].try_into().unwrap()).unwrap();

    let epk = Point::from_bytes(&ca.ephemeral_key.clone().try_into().unwrap())
        .unwrap()
        .to_affine();
    let ka = epk * ivk_fq;
    let key = Params::new()
        .hash_length(32)
        .personal(KDF_ORCHARD_PERSONALIZATION)
        .to_state()
        .update(&ka.to_bytes())
        .update(&ca.ephemeral_key)
        .finalize();
    let mut plaintext = [0; COMPACT_NOTE_SIZE];
    plaintext.copy_from_slice(&ca.ciphertext);
    let mut keystream = ChaCha20::new(key.as_ref().into(), [0u8; 12][..].into());
    keystream.seek(64);
    keystream.apply_keystream(&mut plaintext);

    if (plaintext[0] == 0x01 || plaintext[0] == 0x02)
        && plaintext_version_is_valid(zip212_enforcement, plaintext[0])
    {
        use zcash_note_encryption::Domain;
        let pivk = orchard::keys::PreparedIncomingViewingKey::new(ivk);
        let rho = Rho::from_bytes(&ca.nullifier.clone().try_into().unwrap()).unwrap();
        let cca = CompactAction::from_parts(
            Nullifier::from_bytes(&rho.to_bytes()).unwrap(),
            ExtractedNoteCommitment::from_bytes(&ca.cmx.clone().try_into().unwrap()).unwrap(),
            EphemeralKeyBytes(ca.ephemeral_key.clone().try_into().unwrap()),
            ca.ciphertext.clone().try_into().unwrap(),
        );
        let d = OrchardDomain::for_compact_action(&cca);
        if let Some((note, recipient)) = d.parse_note_plaintext_without_memo_ivk(&pivk, &plaintext)
        {
            let cmx = ExtractedNoteCommitment::from(note.commitment());
            let value = note.value().inner();
            if cmx.to_bytes() == *ca.cmx {
                let dbn = Note {
                    pool: 2,
                    account,
                    scope,
                    height,
                    value,
                    rcm: note.rseed().as_bytes().to_vec(),
                    rho: rho.to_bytes().to_vec(),
                    vout,
                    diversifier: recipient.diversifier().as_array().to_vec(),
                    ivtx,
                    cmx: cmx.to_bytes().to_vec(),
                    ..Note::default()
                };
                return Ok(Some((note, dbn)));
            }
        }
    }
    Ok(None)
}
