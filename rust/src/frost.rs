use anyhow::Result;
use bip39::Mnemonic;
use tracing::info;

use crate::{
    account::get_birth_height,
    api::{
        account::{new_account, NewAccount},
        frost::DKGState,
    },
    get_coin,
};

impl DKGState {
    pub fn seed(&self) -> Mnemonic {
        let mut state = blake2b_simd::Params::new()
            .hash_length(32)
            .personal(b"Zcash__FROST_DKG")
            .to_state();
        for a in self.package.addresses.iter() {
            state.update(a.as_bytes());
        }
        let hash = state.finalize();
        Mnemonic::from_entropy(hash.as_ref()).expect("Failed to create mnemonic from hash")
    }

    pub async fn create_broadcast_account(&mut self, seed: &str) -> Result<u32> {
        let c = get_coin!();
        let connection = c.get_pool();

        let r: Option<(u32,)> = sqlx::query_as("SELECT id_account FROM accounts WHERE seed = ?1")
            .bind(seed)
            .fetch_optional(connection)
            .await?;
        if let Some((account,)) = r {
            info!("Broadcast account already exists");
            return Ok(account);
        }

        let birth_height = get_birth_height(connection, self.package.mailbox_account).await?;
        let na = NewAccount {
            name: format!("{}-frost-broadcast", self.package.name),
            icon: None,
            restore: false,
            key: seed.to_string(),
            passphrase: None,
            fingerprint: None,
            aindex: 0,
            birth: Some(birth_height),
            use_internal: false,
            internal: true,
        };
        let broadcast_account = new_account(&na).await?;
        self.broadcast_account = broadcast_account;

        Ok(broadcast_account)
    }
}
