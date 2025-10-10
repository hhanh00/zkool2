use sapling_crypto::keys::FullViewingKey;
use tracing::info;

use crate::ledger::{APDUCommand, Device, LedgerError, LedgerResult};

pub async fn get_fvk<D: Device>(ledger: &D, aindex: u32) -> LedgerResult<FullViewingKey> {
    let aindex = aindex | 0x8000_0000u32;
    let res = ledger
        .execute(&APDUCommand {
            cla: 0x85,
            ins: 0xF3,
            p1: 1,
            p2: 0,
            data: aindex.to_le_bytes().to_vec(),
        })
        .await?;
    if res.retcode != 0x9000 {
        return Err(LedgerError::Generic(res.retcode, "get_fvk".into()))
    }
    assert_eq!(res.retcode, 0x9000);
    let fvk = FullViewingKey::read(&*res.data)?;
    Ok(fvk)
}
