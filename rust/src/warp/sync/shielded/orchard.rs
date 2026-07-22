use anyhow::Result;
use orchard::{
    keys::{FullViewingKey, IncomingViewingKey},
    note::{AssetBase, ExtractedNoteCommitment, NoteVersion, RandomSeed, Rho},
    value::NoteValue,
    Address, Note,
};
use sqlx::SqliteConnection;
use crate::keys::ScopeExt;

use crate::{
    lwd::{CompactIssueNote, CompactOrchardAction, CompactTx},
    Hash32,
};
use zcash_trees::{network::Network, types};

use crate::warp::{hasher::OrchardHasher, try_orchard_decrypt};

use super::ShieldedProtocol;

pub struct OrchardProtocol;

impl ShieldedProtocol for OrchardProtocol {
    type Hasher = OrchardHasher;
    type IVK = IncomingViewingKey;
    type NK = FullViewingKey;
    type Note = Note;
    type Spend = CompactOrchardAction;
    type Output = CompactOrchardAction;
    type IssueAuth = ();

    async fn extract_ivk(
        connection: &mut SqliteConnection,
        account: u32,
        scope: u8,
    ) -> Result<Option<(Self::IVK, Self::NK)>> {
        let vk: Option<(Vec<u8>,)> =
            sqlx::query_as("SELECT xvk FROM orchard_accounts WHERE account = ?")
                .bind(account)
                .fetch_optional(&mut *connection)
                .await?;
        let keys = vk.map(|(vk,)| {
            let vk = FullViewingKey::from_bytes(&vk.try_into().unwrap()).unwrap();
            let scope = scope.orchard_scope();
            let ivk = vk.to_ivk(scope);
            (ivk, vk)
        });
        Ok(keys)
    }

    fn extract_inputs(tx: &CompactTx) -> &Vec<Self::Spend> {
        &tx.actions
    }

    fn extract_outputs(tx: &CompactTx) -> &Vec<Self::Output> {
        &tx.actions
    }

    fn extract_nf(i: &Self::Spend) -> Hash32 {
        i.nullifier.clone().try_into().unwrap()
    }

    fn extract_cmx(o: &Self::Output) -> Hash32 {
        o.cmx.clone().try_into().unwrap()
    }

    #[allow(clippy::too_many_arguments)]
    fn try_decrypt(
        network: &Network,
        account: u32,
        scope: u8,
        ivk: &Self::IVK,
        height: u32,
        ivtx: u32,
        vout: u32,
        output: &Self::Output,
    ) -> Result<Option<(Self::Note, types::Note)>> {
        try_orchard_decrypt(network, account, scope, ivk, height, ivtx, vout, output)
    }

    fn compute_issuance_cmx(
        issue_note: &CompactIssueNote,
        asset_base: &AssetBase,
    ) -> Result<Option<Hash32>> {
        let (_, cmx) = construct_issuance_note(issue_note, asset_base)?;
        Ok(Some(cmx))
    }

    #[allow(clippy::too_many_arguments)]
    fn try_decrypt_issuance(
        _network: &Network,
        account: u32,
        scope: u8,
        ivk: &Self::IVK,
        height: u32,
        ivtx: u32,
        vout: u32,
        issue_note: &CompactIssueNote,
        asset_base: &AssetBase,
    ) -> Result<Option<(Self::Note, types::Note)>> {
        let recipient_bytes: [u8; 43] = issue_note.recipient.as_slice().try_into()
            .map_err(|_| anyhow::anyhow!("Invalid issuance note recipient length"))?;
        let parsed_addr = Address::from_raw_address_bytes(&recipient_bytes);
        if parsed_addr.is_none().into() {
            return Ok(None);
        }
        let parsed_addr = parsed_addr.unwrap();
        let d = parsed_addr.diversifier();
        let our_addr = ivk.address(d);
        if our_addr.to_raw_address_bytes() != recipient_bytes {
            return Ok(None);
        }

        let (note, cmx_bytes) = construct_issuance_note(issue_note, asset_base)?;
        let is_zec = bool::from(note.asset().is_zatoshi());
        let value = note.value().inner();
        let rho = note.rho();
        let dbn = types::Note {
            pool: 2, // Orchard
            account,
            scope,
            height,
            value,
            rcm: note.rseed().as_bytes().to_vec(),
            rho: rho.to_bytes().to_vec(),
            vout,
            diversifier: our_addr.diversifier().as_array().to_vec(),
            ivtx,
            cmx: cmx_bytes.to_vec(),
            asset_base: if is_zec { vec![] } else { note.asset().to_bytes().to_vec() },
            ..types::Note::default()
        };
        Ok(Some((note, dbn)))
    }

    fn derive_nf(nk: &Self::NK, _position: u32, note: &mut Self::Note) -> Result<Hash32> {
        let nf = note.nullifier(nk);
        Ok(nf.to_bytes())
    }
}

/// Construct an Orchard note from a plaintext issuance note and return both the
/// note and its extracted cmx (note commitment x-coordinate).
fn construct_issuance_note(
    issue_note: &CompactIssueNote,
    asset_base: &AssetBase,
) -> Result<(Note, Hash32)> {
    let recipient_bytes: [u8; 43] = issue_note.recipient.as_slice().try_into()
        .map_err(|_| anyhow::anyhow!("Invalid issuance note recipient length"))?;
    let addr = Address::from_raw_address_bytes(&recipient_bytes);
    if addr.is_none().into() {
        anyhow::bail!("Invalid issuance note recipient address");
    }
    let addr = addr.unwrap();

    let value = NoteValue::from_raw(issue_note.value);
    let rho = Rho::from_bytes(
        issue_note.rho.as_slice().try_into()
            .map_err(|_| anyhow::anyhow!("Invalid issuance note rho length"))?,
    );
    if rho.is_none().into() {
        anyhow::bail!("Invalid issuance note rho");
    }
    let rho = rho.unwrap();
    let rseed = RandomSeed::from_bytes(
        issue_note.rseed.as_slice().try_into()
            .map_err(|_| anyhow::anyhow!("Invalid issuance note rseed length"))?,
        &rho,
    );
    if rseed.is_none().into() {
        anyhow::bail!("Invalid issuance note rseed");
    }
    let rseed = rseed.unwrap();

    let note = Note::from_parts(addr, value, *asset_base, rho, rseed, NoteVersion::V2);
    if note.is_none().into() {
        anyhow::bail!("Invalid issuance note");
    }
    let note = note.unwrap();
    let cmx = ExtractedNoteCommitment::from(note.commitment());
    Ok((note, cmx.to_bytes()))
}
