use anyhow::Result;
use orchard::{
    keys::{FullViewingKey, IncomingViewingKey},
    Note,
};
use sqlx::SqliteConnection;

use crate::{
    lwd::CompactOrchardAction,
    warp::sync::block::SyncTx,
    Hash32,
};
use zcash_trees::{network::Network, types};

use crate::warp::{hasher::OrchardHasher, try_orchard_decrypt};

use super::ShieldedProtocol;

pub struct IronwoodProtocol;

impl ShieldedProtocol for IronwoodProtocol {
    type Hasher = OrchardHasher;
    type IVK = IncomingViewingKey;
    type NK = FullViewingKey;
    type Note = Note;
    type Spend = CompactOrchardAction;
    type Output = CompactOrchardAction;
    type IssueAuth = ();

    fn supports_issuance() -> bool {
        false
    }

    async fn extract_ivk(
        connection: &mut SqliteConnection,
        account: u32,
        scope: u8,
    ) -> Result<Option<(Self::IVK, Self::NK)>> {
        // Ironwood uses the same keys as Orchard
        let vk: Option<(Vec<u8>,)> =
            sqlx::query_as("SELECT xvk FROM orchard_accounts WHERE account = ?")
                .bind(account)
                .fetch_optional(&mut *connection)
                .await?;
        let keys = vk.map(|(vk,)| {
            let vk = FullViewingKey::from_bytes(&vk.try_into().unwrap()).unwrap();
            let scope = if scope == 0 {
                orchard::keys::Scope::External
            } else {
                orchard::keys::Scope::Internal
            };
            let ivk = vk.to_ivk(scope);
            (ivk, vk)
        });
        Ok(keys)
    }

    async fn extract_issue_auth(
        _connection: &mut SqliteConnection,
        _account: u32,
        _coin_type: u32,
    ) -> Result<Option<(Self::IssueAuth, Self::NK)>> {
        Ok(None)
    }

    fn extract_inputs(tx: &SyncTx) -> &Vec<Self::Spend> {
        &tx.ironwood_actions
    }

    fn extract_outputs(tx: &SyncTx) -> &Vec<Self::Output> {
        &tx.ironwood_actions
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
        // Ironwood uses the exact same note encryption as Orchard.
        try_orchard_decrypt(network, account, scope, ivk, height, ivtx, vout, output)
    }

    fn derive_nf(nk: &Self::NK, _position: u32, note: &mut Self::Note) -> Result<Hash32> {
        let nf = note.nullifier(nk);
        Ok(nf.to_bytes())
    }
}
