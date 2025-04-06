use anyhow::Result;
use sapling_crypto::{
    zip32::DiversifiableFullViewingKey, Note, NullifierDerivingKey, SaplingIvk
};
use sqlx::{Pool, Sqlite};
use zcash_protocol::consensus::Network;
use zcash_primitives::zip32::Scope;

use crate::{
    lwd::{CompactSaplingOutput, CompactSaplingSpend, CompactTx}, warp::{hasher::SaplingHasher, try_sapling_decrypt}, Hash32
};

use super::ShieldedProtocol;

pub struct SaplingProtocol;

impl ShieldedProtocol for SaplingProtocol {
    type Hasher = SaplingHasher;
    type IVK = SaplingIvk;
    type NK = NullifierDerivingKey;
    type Note = Note;
    type Spend = CompactSaplingSpend;
    type Output = CompactSaplingOutput;

    fn extract_inputs(tx: &CompactTx) -> &Vec<Self::Spend> {
        &tx.spends
    }

    fn extract_outputs(tx: &CompactTx) -> &Vec<Self::Output> {
        &tx.outputs
    }

    fn extract_nf(i: &Self::Spend) -> Hash32 {
        i.clone().nf.try_into().unwrap()
    }

    fn extract_cmx(o: &Self::Output) -> Hash32 {
        o.cmu.clone().try_into().unwrap()
    }

    async fn extract_ivk(
        connection: &Pool<Sqlite>,
        account: u32,
    ) -> Result<Option<(Self::IVK, Self::NK)>> {
        let vk: Option<(Vec<u8>, )> = sqlx::query_as("SELECT xvk FROM sapling_accounts WHERE account = ?")
            .bind(account)
            .fetch_optional(connection)
            .await?;
        let keys = vk.map(|(vk, )| {
            let vk = DiversifiableFullViewingKey::from_bytes(&vk.try_into().unwrap()).unwrap();
            let ivk = vk.fvk().vk.ivk();
            let nk = vk.to_nk(Scope::External);
            (ivk, nk)
        });
        Ok(keys)
    }

    fn try_decrypt(
        network: &Network,
        account: u32,
        ivk: &Self::IVK,
        height: u32,
        ivtx: u32,
        vout: u32,
        output: &Self::Output,
    ) -> Result<Option<(sapling_crypto::Note, crate::sync::Note)>> {
        try_sapling_decrypt(network, account, ivk, height, ivtx, vout, output)
    }
    
    fn derive_nf(nk: &Self::NK, position: u32, note: &mut Self::Note) -> Result<Hash32> {
        let nf = note.nf(nk, position as u64);
        Ok(nf.0)
    }
}
