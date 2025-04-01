use anyhow::Result;
use zcash_address::unified::{Encoding as _, Fvk, Ufvk};
use zcash_keys::keys::UnifiedFullViewingKey;

use crate::db::{select_account_orchard, select_account_sapling, select_account_transparent};

pub fn get_account_ufvk() -> Result<UnifiedFullViewingKey> {
    let tkeys = select_account_transparent()?;
    let skeys = select_account_sapling()?;
    let okeys = select_account_orchard()?;

    let items = vec![
        tkeys.xvk.map(|vk| Fvk::P2pkh(vk.serialize().try_into().unwrap())),
        skeys.xvk.map(|vk| Fvk::Sapling(vk.to_bytes())),
        okeys.xvk.map(|vk| Fvk::Orchard(vk.to_bytes())),
        ];
    let items = items.into_iter().filter_map(|x| x).collect::<Vec<Fvk>>();

    let ufvk = Ufvk::try_from_items(items)?;
    let ufvk = UnifiedFullViewingKey::parse(&ufvk)?;

    Ok(ufvk)
}
