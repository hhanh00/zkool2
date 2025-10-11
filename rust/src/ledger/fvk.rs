use std::io::Write;

use sapling_crypto::{keys::FullViewingKey, PaymentAddress};
use zcash_keys::encoding::AddressCodec;

use crate::{
    coin::Network,
    ledger::{connect_ledger, APDUCommand, Device, LedgerError, LedgerResult},
    tiu,
};

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
        return Err(LedgerError::Generic(res.retcode, "get_fvk".into()));
    }
    assert_eq!(res.retcode, 0x9000);
    let fvk = FullViewingKey::read(&*res.data)?;
    Ok(fvk)
}

pub async fn get_next_diversifier_address(
    network: &Network,
    aindex: u32,
    dindex: u32,
) -> LedgerResult<(u32, String)> {
    let ledger = connect_ledger().await?;
    let mut data = vec![];
    let aindex = aindex | 0x8000_0000u32;
    data.write_all(&aindex.to_le_bytes())?;
    data.write_all(&dindex.to_le_bytes())?;
    data.write_all(&[0u8; 7])?; // div index is 11 bytes (4 + 7)
    assert_eq!(data.len(), 15);

    let get_div_list = APDUCommand {
        cla: 0x85,
        ins: 0x09,
        p1: 0,
        p2: 0,
        data,
    };
    let res = ledger.execute(&get_div_list).await?;
    if res.retcode != 0x9000 {
        return Err(LedgerError::Execute(res.retcode, get_div_list.ins));
    }
    for i in 0..20 {
        let div = &res.data[i * 11..(i + 1) * 11];
        if div != [0u8; 11] {
            let dindex = dindex + i as u32;
            let mut data = vec![];
            data.write_all(&aindex.to_le_bytes())?;
            data.write_all(div)?;
            let get_address_div = APDUCommand {
                cla: 0x85,
                ins: 0x10,
                p1: 1,
                p2: 0,
                data,
            };
            let res = ledger.execute(&get_address_div).await?;
            if res.retcode != 0x9000 {
                return Err(LedgerError::Execute(res.retcode, get_address_div.ins));
            }
            let address = &res.data[0..43];
            let address = PaymentAddress::from_bytes(tiu!(address)).unwrap();
            let address = address.encode(network);
            return Ok((dindex, address));
        }
    }
    Err(LedgerError::Anyhow(anyhow::anyhow!(
        "No diversified addresses found"
    )))
}

#[cfg(test)]
mod tests {
    use crate::ledger::{APDUCommand, Device, LEDGER_ZEMU};
    use std::io::Write;

    #[tokio::test]
    pub async fn f() -> anyhow::Result<()> {
        let ledger = LEDGER_ZEMU.lock().await;
        let aindex = 0x8000_0000u32;
        let dindex = 0u32;
        let mut data = vec![];
        data.write_all(&aindex.to_le_bytes())?;
        data.write_all(&dindex.to_le_bytes())?;
        data.write_all(&[0u8; 7])?; // div index is 11 bytes (4 + 7)
        let get_address_div = APDUCommand {
            cla: 0x85,
            ins: 0x09,
            p1: 0,
            p2: 0,
            data,
        };

        let res = ledger.execute(&get_address_div).await?;
        println!("{}", res.retcode);
        Ok(())
    }
}
