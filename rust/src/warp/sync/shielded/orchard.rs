use anyhow::Result;
use orchard::{
    keys::{FullViewingKey, IncomingViewingKey, Scope},
    Note,
};
use sqlx::SqlitePool;
use zcash_protocol::consensus::Network;

use crate::{
    lwd::{CompactOrchardAction, CompactTx},
    warp::{hasher::OrchardHasher, try_orchard_decrypt}, Hash32,
};

use super::ShieldedProtocol;

pub struct OrchardProtocol;

impl ShieldedProtocol for OrchardProtocol {
    type Hasher = OrchardHasher;
    type IVK = IncomingViewingKey;
    type NK = FullViewingKey;
    type Spend = CompactOrchardAction;
    type Output = CompactOrchardAction;
    type Note = Note;

    async fn extract_ivk(connection: &SqlitePool, account: u32, scope: u8) -> Result<Option<(Self::IVK, Self::NK)>> {
        let vk: Option<(Vec<u8>, )> = sqlx::query_as("SELECT xvk FROM orchard_accounts WHERE account = ?")
            .bind(account)
            .fetch_optional(connection)
            .await?;
        let keys = vk.map(|(vk, )| {
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

    fn try_decrypt(
        network: &Network,
        account: u32,
        scope: u8,
        ivk: &Self::IVK,
        height: u32,
        ivtx: u32,
        vout: u32,
        output: &Self::Output,
    ) -> Result<Option<(Self::Note, crate::sync::Note)>> {
        try_orchard_decrypt(network, account, scope, ivk, height, ivtx, vout, output)
    }

    fn derive_nf(nk: &Self::NK, _position: u32, note: &mut Self::Note) -> Result<crate::Hash32> {
        let nf = note.nullifier(nk);
        Ok(nf.to_bytes())
    }
}
