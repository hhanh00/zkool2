use anyhow::Result;

#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

use crate::api::coin::Coin;
use crate::db::ZsaAssetRow;

/// A ZSA token holding representing a balance of a specific asset.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug)]
pub struct ZsaHolding {
    pub id_asset: i64,
    pub asset_desc_hash: Vec<u8>,
    pub asset_name: String,
    pub ik: Vec<u8>,
    pub asset_base: Vec<u8>,
    pub finalized: bool,
    pub first_seen_height: u32,
    pub balance: u64,
}

impl From<ZsaAssetRow> for ZsaHolding {
    fn from(r: ZsaAssetRow) -> Self {
        ZsaHolding {
            id_asset: r.id_asset,
            asset_desc_hash: r.asset_desc_hash,
            asset_name: r.asset_name.unwrap_or_default(),
            ik: r.ik,
            asset_base: r.asset_base,
            finalized: r.finalized,
            first_seen_height: r.first_seen_height as u32,
            balance: r.balance as u64,
        }
    }
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn list_zsa_holdings(c: &Coin) -> Result<Vec<ZsaHolding>> {
    let mut connection = c.get_connection().await?;
    let rows = crate::db::get_zsa_holdings(&mut connection, c.account).await?;
    Ok(rows.into_iter().map(ZsaHolding::from).collect())
}
