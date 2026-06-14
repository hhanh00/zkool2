use crate::lwd::{CompactOrchardAction, CompactSaplingOutput};
use zcash_protocol::consensus::ZIP212_GRACE_PERIOD;
use zcash_trees::network::Network;
use zcash_trees::types::Note;

use anyhow::Result;
use blake2b_simd::Params;
use tracing::info;
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
    primitives::{CompactAction, OrchardDomain},
    flavor::{OrchardVanilla, OrchardZSA},
};
use sapling_crypto::{
    note_encryption::{
        plaintext_version_is_valid, SaplingDomain, Zip212Enforcement, KDF_SAPLING_PERSONALIZATION,
    },
    SaplingIvk,
};
use zcash_note_encryption::{note_bytes::{NoteBytesData, NoteBytes}, EphemeralKeyBytes};

const COMPACT_NOTE_SIZE: usize = 52;

fn zip212(network: &Network, height: u32) -> Zip212Enforcement {
    use zcash_protocol::consensus::{BlockHeight, NetworkUpgrade, Parameters};
    let height = BlockHeight::from_u32(height);
    match network.activation_height(NetworkUpgrade::Canopy) {
        Some(h) => {
            if height >= h + ZIP212_GRACE_PERIOD {
                Zip212Enforcement::On
            } else if height >= h {
                Zip212Enforcement::GracePeriod
            } else {
                Zip212Enforcement::Off
            }
        }
        _ => Zip212Enforcement::On, // this doesn't apply for Zcash
    }
}

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
) -> Result<Option<(sapling_crypto::Note, Note)>> {
    let epkb = &*co.epk;
    let epk = jubjub::AffinePoint::from_bytes(epkb.try_into().unwrap()).unwrap();
    let enc = &co.ciphertext;
    let epk = epk.mul_by_cofactor().to_niels();
    let zip212_enforcement = zip212(network, height);
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
        if let Some((note, recipient)) = d.parse_note_plaintext_without_memo_ivk(&pivk, &NoteBytesData(plaintext))
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
    let zip212_enforcement = zip212(network, height);
    let bb = ivk.to_bytes();
    let ivk_bytes: [u8; 32] = bb[32..64].try_into()
        .map_err(|_| anyhow::anyhow!("Invalid ivk length"))?;
    let ivk_fq = Option::<Fq>::from(Fq::from_repr(ivk_bytes))
        .ok_or_else(|| anyhow::anyhow!("Invalid ivk Fq repr"))?;

    let epk_bytes: [u8; 32] = ca.ephemeral_key.clone().try_into()
        .map_err(|_| anyhow::anyhow!("Invalid ephemeral key length"))?;
    let epk = Option::<Point>::from(Point::from_bytes(&epk_bytes))
        .ok_or_else(|| anyhow::anyhow!("Invalid ephemeral key bytes"))?
        .to_affine();
    let ka = epk * ivk_fq;
    let key = Params::new()
        .hash_length(32)
        .personal(KDF_ORCHARD_PERSONALIZATION)
        .to_state()
        .update(&ka.to_bytes())
        .update(&ca.ephemeral_key)
        .finalize();
    let ciphertext_len = ca.ciphertext.len();
    let mut keystream = ChaCha20::new(key.as_ref().into(), [0u8; 12][..].into());
    keystream.seek(64);

    // Handle ZSA (84-byte) and vanilla (52-byte) ciphertext sizes
    if ciphertext_len >= 84 {
        // ZSA ciphertext can be > 84 bytes; use a dynamic buffer
        let mut plaintext = ca.ciphertext.clone();
        keystream.apply_keystream(&mut plaintext);

        // ZSA notes use leadbyte 0x03, vanilla uses 0x01/0x02
        let is_zsa = plaintext[0] == 0x03;
        if (is_zsa || plaintext[0] == 0x01 || plaintext[0] == 0x02)
            && (is_zsa || plaintext_version_is_valid(zip212_enforcement, plaintext[0]))
        {
            use zcash_note_encryption::Domain;
            let pivk = orchard::keys::PreparedIncomingViewingKey::new(ivk);
            let nullifier_bytes: [u8; 32] = ca.nullifier.clone()
                .try_into()
                .map_err(|_| anyhow::anyhow!("Invalid nullifier length"))?;
            let rho = Option::<Rho>::from(Rho::from_bytes(&nullifier_bytes))
                .ok_or_else(|| anyhow::anyhow!("Invalid Rho bytes"))?;
            let note_ciphertext = NoteBytesData::from_slice(&ca.ciphertext)
                .ok_or_else(|| anyhow::anyhow!("Invalid orchard note ciphertext"))?;
            let cmx_bytes: [u8; 32] = ca.cmx.clone()
                .try_into()
                .map_err(|_| anyhow::anyhow!("Invalid cmx length"))?;
            let ephemeral_key_bytes: [u8; 32] = ca.ephemeral_key.clone()
                .try_into()
                .map_err(|_| anyhow::anyhow!("Invalid ephemeral key length"))?;
            let cca = CompactAction::<OrchardZSA>::from_parts(
                Option::<Nullifier>::from(Nullifier::from_bytes(&rho.to_bytes()))
                    .ok_or_else(|| anyhow::anyhow!("Invalid nullifier"))?,
                Option::<ExtractedNoteCommitment>::from(
                    ExtractedNoteCommitment::from_bytes(&cmx_bytes)
                ).ok_or_else(|| anyhow::anyhow!("Invalid cmx"))?,
                EphemeralKeyBytes(ephemeral_key_bytes),
                note_ciphertext,
            );
            let d = OrchardDomain::<OrchardZSA>::for_compact_action(&cca);
            let note_plaintext = NoteBytesData::from_slice(&plaintext)
                .ok_or_else(|| anyhow::anyhow!("Invalid orchard note plaintext"))?;
            if let Some((note, recipient)) = d.parse_note_plaintext_without_memo_ivk(
                &pivk,
                &note_plaintext,
            ) {
                let cmx = ExtractedNoteCommitment::from(note.commitment());
                let value = note.value().inner();
                if cmx.to_bytes() == *ca.cmx {
                    // Always 32 bytes: [0u8; 32] for ZEC, non-zero for ZSA
                    let asset_base = note.asset().to_bytes().to_vec();
                    if is_zsa {
                        info!(
                            "ZSA note: account={account} scope={scope} height={height} value={value} asset={asset:?}",
                            asset = hex::encode(&asset_base)
                        );
                    }
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
                        asset_base,
                        ..Note::default()
                    };
                    return Ok(Some((note, dbn)));
                }
            }
        }
        return Ok(None);
    }

    // Vanilla (52-byte) path
    let mut plaintext = [0u8; 52];
    plaintext.copy_from_slice(&ca.ciphertext);
    keystream.apply_keystream(&mut plaintext);

    if (plaintext[0] == 0x01 || plaintext[0] == 0x02)
        && plaintext_version_is_valid(zip212_enforcement, plaintext[0])
    {
        use zcash_note_encryption::Domain;
        let pivk = orchard::keys::PreparedIncomingViewingKey::new(ivk);
        let nullifier_bytes: [u8; 32] = ca.nullifier.clone()
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid nullifier length"))?;
        let rho = Option::<Rho>::from(Rho::from_bytes(&nullifier_bytes))
            .ok_or_else(|| anyhow::anyhow!("Invalid Rho bytes"))?;
        let note_ciphertext = NoteBytesData::from_slice(&ca.ciphertext)
            .ok_or_else(|| anyhow::anyhow!("Invalid orchard note ciphertext"))?;
        let cmx_bytes: [u8; 32] = ca.cmx.clone()
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid cmx length"))?;
        let ephemeral_key_bytes: [u8; 32] = ca.ephemeral_key.clone()
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid ephemeral key length"))?;
        let cca = CompactAction::<OrchardVanilla>::from_parts(
            Option::<Nullifier>::from(Nullifier::from_bytes(&rho.to_bytes()))
                .ok_or_else(|| anyhow::anyhow!("Invalid nullifier"))?,
            Option::<ExtractedNoteCommitment>::from(
                ExtractedNoteCommitment::from_bytes(&cmx_bytes)
            ).ok_or_else(|| anyhow::anyhow!("Invalid cmx"))?,
            EphemeralKeyBytes(ephemeral_key_bytes),
            note_ciphertext,
        );
        let d = OrchardDomain::<OrchardVanilla>::for_compact_action(&cca);
        if let Some((note, recipient)) = d.parse_note_plaintext_without_memo_ivk(
            &pivk,
            &NoteBytesData(plaintext),
        )
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
                    asset_base: note.asset().to_bytes().to_vec(),
                    ..Note::default()
                };
                return Ok(Some((note, dbn)));
            }
        }
    }
    Ok(None)
}
