use anyhow::Result;
use orchard::{
    keys::{FullViewingKey, IncomingViewingKey, Scope},
    note::{AssetBase, RandomSeed, Rho},
    issuance::auth::{IssueAuthKey, IssueValidatingKey, ZSASchnorr},
    value::NoteValue,
    Address, Note,
};
use sqlx::SqliteConnection;

use crate::{
    lwd::CompactOrchardAction,
    warp::sync::block::{OrchardOutput, SyncTx},
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
    type Output = OrchardOutput;
    type IssueAuth = IssueValidatingKey<ZSASchnorr>;

    fn supports_issuance() -> bool {
        true
    }

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
            let scope = if scope == 1 {
                Scope::Internal
            } else {
                Scope::External
            };
            let ivk = vk.to_ivk(scope);
            (ivk, vk)
        });
        Ok(keys)
    }

    async fn extract_issue_auth(
        connection: &mut SqliteConnection,
        account: u32,
        coin_type: u32,
    ) -> Result<Option<(Self::IssueAuth, Self::NK)>> {
        if let Ok(Some(seed_info)) =
            crate::account::get_account_seed(&mut *connection, account).await
        {
            if let Ok(mnemonic) = bip39::Mnemonic::parse(seed_info.mnemonic) {
                let seed = mnemonic.to_seed(&seed_info.phrase);
                if let Ok(isk) =
                    IssueAuthKey::<ZSASchnorr>::from_zip32_seed(&seed, coin_type, 0)
                {
                    let ik = IssueValidatingKey::from(&isk);
                    // Reuse the FVK from orchard_accounts for nullifier derivation
                    let vk: Option<(Vec<u8>,)> = sqlx::query_as(
                        "SELECT xvk FROM orchard_accounts WHERE account = ?",
                    )
                    .bind(account)
                    .fetch_optional(&mut *connection)
                    .await?;
                    if let Some((xvk,)) = vk {
                        let fvk = FullViewingKey::from_bytes(&xvk.try_into().unwrap())
                            .unwrap();
                        return Ok(Some((ik, fvk)));
                    }
                }
            }
        }
        Ok(None)
    }

    fn extract_inputs(tx: &SyncTx) -> &Vec<Self::Spend> {
        &tx.orchard_actions
    }

    fn extract_outputs(tx: &SyncTx) -> &Vec<Self::Output> {
        &tx.orchard_outputs
    }

    fn extract_nf(i: &Self::Spend) -> Hash32 {
        i.nullifier.clone().try_into().unwrap()
    }

    fn extract_cmx(o: &Self::Output) -> Hash32 {
        match o {
            OrchardOutput::Action(a) => a.cmx.clone().try_into().unwrap(),
            OrchardOutput::Issuance { cmx, .. } => *cmx,
        }
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
        match output {
            OrchardOutput::Action(a) => {
                try_orchard_decrypt(network, account, scope, ivk, height, ivtx, vout, a)
            }
            OrchardOutput::Issuance {
                note: note_data,
                asset_base,
                cmx,
                owner,
                ..
            } => {
                // Only synthesize if this account owns the issuance key and
                // we are on the external scope (issuance uses Scope::External)
                if *owner != Some(account) || scope != 0 {
                    return Ok(None);
                }
                let recipient_bytes: [u8; 43] =
                    note_data.recipient.as_slice().try_into().unwrap();
                let recipient =
                    Address::from_raw_address_bytes(&recipient_bytes).unwrap();
                let rho_bytes: [u8; 32] =
                    note_data.rho.as_slice().try_into().unwrap();
                let rho = Rho::from_bytes(&rho_bytes).unwrap();
                let rseed_bytes: [u8; 32] =
                    note_data.rseed.as_slice().try_into().unwrap();
                let rseed = RandomSeed::from_bytes(rseed_bytes, &rho).unwrap();
                let asset_base_bytes: [u8; 32] =
                    asset_base.as_slice().try_into().unwrap();
                let asset_base_val =
                    AssetBase::from_bytes(&asset_base_bytes).unwrap();

                let note = Note::from_parts(
                    recipient,
                    NoteValue::from_raw(note_data.value),
                    asset_base_val,
                    rho,
                    rseed,
                )
                .unwrap();

                let dbn = types::Note {
                    account,
                    scope: 0,
                    height,
                    pool: 2,
                    value: note_data.value,
                    cmx: cmx.to_vec(),
                    asset_base: asset_base.clone(),
                    rho: note_data.rho.clone(),
                    rcm: note.rseed().as_bytes().to_vec(),
                    diversifier: recipient.diversifier().as_array().to_vec(),
                    ivtx,
                    vout,
                    ..types::Note::default()
                };

                Ok(Some((note, dbn)))
            }
        }
    }

    fn derive_nf(nk: &Self::NK, _position: u32, note: &mut Self::Note) -> Result<Hash32> {
        let nf = note.nullifier(nk);
        Ok(nf.to_bytes())
    }
}
