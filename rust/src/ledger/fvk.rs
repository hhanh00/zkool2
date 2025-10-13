use std::io::Write;

use byteorder::{WriteBytesExt, LE};
use sapling_crypto::{keys::FullViewingKey, PaymentAddress};
use secp256k1::PublicKey;
use sqlx::SqliteConnection;
use zcash_keys::encoding::AddressCodec;
use zcash_primitives::legacy::TransparentAddress;
use zcash_protocol::consensus::NetworkConstants;

use crate::{
    coin::Network, db::{get_account_aindex, get_account_dindex}, ledger::{connect_ledger, APDUCommand, Device, LedgerError, LedgerResult}, tiu, IntoAnyhow
};

pub async fn get_fvk<D: Device>(ledger: &D, aindex: u32) -> LedgerResult<FullViewingKey> {
    let aindex = aindex | 0x8000_0000u32;
    let res = ledger
        .execute(APDUCommand {
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

pub async fn get_hw_next_diversifier_address(
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
    let res = ledger.execute(get_div_list).await?;
    if res.retcode != 0x9000 {
        return Err(LedgerError::Execute(res.retcode, 0x09));
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
            let res = ledger.execute(get_address_div).await?;
            if res.retcode != 0x9000 {
                return Err(LedgerError::Execute(res.retcode, 0x10));
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

pub async fn get_hw_transparent_address(
    network: &Network,
    aindex: u32,
    scope: u32,
    dindex: u32,
) -> LedgerResult<(Vec<u8>, TransparentAddress)> {
    let ledger = connect_ledger().await?;
    let mut data = vec![];
    let coin_type = network.coin_type();
    data.write_all(&(0x8000_0000u32 | 44).to_le_bytes())?;
    data.write_all(&(0x8000_0000u32 | coin_type).to_le_bytes())?;
    data.write_all(&(0x8000_0000u32 | aindex).to_le_bytes())?;
    data.write_all(&(scope).to_le_bytes())?;
    data.write_all(&(dindex).to_le_bytes())?;
    assert_eq!(data.len(), 20);
    let get_taddress = APDUCommand {
        cla: 0x85,
        ins: 0x01,
        p1: 0,
        p2: 0,
        data,
    };
    let res = ledger.execute(get_taddress).await?;
    if res.retcode != 0x9000 {
        return Err(LedgerError::Execute(res.retcode, 0x01));
    }
    let pk = &res.data[0..33];
    let pubkey = PublicKey::from_slice(pk).anyhow()?;
    let taddress = TransparentAddress::from_pubkey(&pubkey);
    Ok((pk.to_vec(), taddress))
}

pub async fn show_sapling_address(network: &Network, connection: &mut SqliteConnection, account: u32) -> LedgerResult<String> {
    let ledger = connect_ledger().await?;
    let aindex = get_account_aindex(connection, account).await? | 0x80000000u32;
    let dindex = get_account_dindex(connection, account).await?;
    let mut data = vec![];
    data.write_u32::<LE>(aindex)?;
    data.write_u32::<LE>(dindex)?;
    data.write_all(&[0u8; 7])?;
    let get_div = APDUCommand {
        cla: 0x85,
        ins: 0x09,
        p1: 0,
        p2: 0,
        data,
    };
    let res = ledger.execute(get_div).await?;
    if res.retcode != 0x9000 {
        return Err(LedgerError::Execute(res.retcode, 0x09));
    }
    let div = &res.data[0..11];
    let mut data = vec![];
    data.write_u32::<LE>(aindex)?;
    data.write_all(div)?;
    let get_address = APDUCommand {
        cla: 0x85,
        ins: 0x10,
        p1: 1,
        p2: 0,
        data,
    };
    let res = ledger.execute(get_address).await?;
    if res.retcode != 0x9000 {
        return Err(LedgerError::Execute(res.retcode, 0x10));
    }
    let address: [u8; 43] = tiu!(&res.data[0..43]);
    let address = PaymentAddress::from_bytes(&address).unwrap();
    Ok(address.encode(network))
}

pub async fn show_transparent_address(network: &Network, connection: &mut SqliteConnection, account: u32) -> LedgerResult<String> {
    let ledger = connect_ledger().await?;
    let aindex = get_account_aindex(connection, account).await?;
    let dindex = get_account_dindex(connection, account).await?;
    let mut data = vec![];
    data.write_u32::<LE>(0x80000000u32 | 44)?;
    data.write_u32::<LE>(0x80000000u32 | network.coin_type())?;
    data.write_u32::<LE>(0x80000000u32 | aindex)?;
    data.write_u32::<LE>(0)?;
    data.write_u32::<LE>(dindex)?;
    let get_address = APDUCommand {
        cla: 0x85,
        ins: 0x01,
        p1: 1,
        p2: 0,
        data,
    };
    let res = ledger.execute(get_address).await?;
    if res.retcode != 0x9000 {
        return Err(LedgerError::Execute(res.retcode, 0x01));
    }
    let pk = &res.data[0..33];
    let pk = PublicKey::from_slice(pk).anyhow()?;
    let ta = TransparentAddress::from_pubkey(&pk);
    Ok(ta.encode(network))
}

#[cfg(test)]
mod tests {
    use crate::ledger::{APDUCommand, Device, LEDGER_ZEMU};
    use std::io::Write;

    #[tokio::test]
    pub async fn f() -> anyhow::Result<()> {
        let ledger = LEDGER_ZEMU.lock().await.clone().unwrap();
        // let ledger = connect_ledger().await?;
        let aindex = 0u32;
        let dindex = 0u32;
        let mut data = vec![];
        data.write_all(&(0x8000_0000u32 | 44).to_le_bytes())?;
        data.write_all(&(0x8000_0000u32 | 133).to_le_bytes())?;
        data.write_all(&(0x8000_0000u32 | aindex).to_le_bytes())?;
        data.write_all(&(aindex).to_le_bytes())?;
        data.write_all(&(dindex).to_le_bytes())?;
        let get_taddress = APDUCommand {
            cla: 0x85,
            ins: 0x01,
            p1: 1,
            p2: 0,
            data,
        };

        println!("{}", get_taddress.ins);
        let res = ledger.execute(get_taddress).await?;
        println!("{}", res.retcode);
        let address = String::from_utf8(res.data[33..].to_vec())?;
        println!("{address}");
        Ok(())
    }
}
