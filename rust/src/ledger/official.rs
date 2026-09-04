// Official Ledger App (LedgerHQ/app-zcash, CLA 0xE0)

use anyhow::Result;
use byteorder::{WriteBytesExt, BE};
use tonic::async_trait;
use zcash_keys::keys::UnifiedFullViewingKey;
use zcash_protocol::consensus::NetworkConstants as _;

use crate::{
    api::coin::Network,
    ledger::{
        transport::{connect_ledger, APDUCommand, Device},
        HwKind, LedgerApp, LedgerError, LedgerResult,
    },
};

pub struct OfficialApp {}

const CLA: u8 = 0xE0;
const INS_GET_VK: u8 = 0x50;
const P1_FIRST: u8 = 0x00;
const P1_CONTINUE: u8 = 0x80;
const P2_UFVK: u8 = 0x00;
const SW_OK: u16 = 0x9000;
const SW_DENY: u16 = 0x6985;
const HARDENED: u32 = 0x8000_0000;
const MAX_RESPONSE_LEN: usize = 4096;

fn append_path(data: &mut Vec<u8>, purpose: u32, coin_type: u32, account: u32) -> LedgerResult<()> {
    data.write_u8(3)?;
    data.write_u32::<BE>(purpose | HARDENED)?;
    data.write_u32::<BE>(coin_type | HARDENED)?;
    data.write_u32::<BE>(account | HARDENED)?;
    Ok(())
}

/// Ask the device for the account UFVK (Orchard + transparent receivers).
/// The user must approve the export on the device.
pub async fn get_ufvk<D: Device>(ledger: &D, network: &Network, aindex: u32) -> LedgerResult<String> {
    let coin_type = network.coin_type();
    let mut data = vec![];
    // m/32'/coin'/account' (Orchard, ZIP-32) then m/44'/coin'/account' (transparent)
    append_path(&mut data, 32, coin_type, aindex)?;
    append_path(&mut data, 44, coin_type, aindex)?;
    assert_eq!(data.len(), 26);

    let get_vk = APDUCommand {
        cla: CLA,
        ins: INS_GET_VK,
        p1: P1_FIRST,
        p2: P2_UFVK,
        data,
    };
    let res = ledger.execute(get_vk).await?;
    if res.retcode == SW_DENY {
        return Err(LedgerError::Generic(
            SW_DENY,
            "user refused to export the viewing key".into(),
        ));
    }
    if res.retcode != SW_OK {
        return Err(LedgerError::Execute(res.retcode, INS_GET_VK));
    }

    // response is len (u16 BE) || ufvk string, delivered in chunks
    let mut payload = res.data;
    if payload.len() < 2 {
        return Err(LedgerError::Protocol("short vk response".into()));
    }
    let len = u16::from_be_bytes([payload[0], payload[1]]) as usize;
    payload.drain(..2);
    if len > payload.len() {
        if len > MAX_RESPONSE_LEN {
            return Err(LedgerError::Protocol("vk response too long".into()));
        }
        payload.reserve(len - payload.len());
    }
    while payload.len() < len {
        let next = APDUCommand {
            cla: CLA,
            ins: INS_GET_VK,
            p1: P1_CONTINUE,
            p2: P2_UFVK,
            data: vec![],
        };
        let res = ledger.execute(next).await?;
        if res.retcode != SW_OK {
            return Err(LedgerError::Execute(res.retcode, INS_GET_VK));
        }
        payload.extend_from_slice(&res.data);
    }
    payload.truncate(len);

    let ufvk = String::from_utf8(payload).map_err(|_| LedgerError::Protocol("invalid utf8 in vk response".into()))?;
    let uvk = UnifiedFullViewingKey::decode(network, &ufvk)
        .map_err(|_| LedgerError::Protocol("device returned an invalid UFVK".into()))?;
    if uvk.orchard().is_none() || uvk.transparent().is_none() {
        return Err(LedgerError::Protocol(
            "device UFVK is missing the orchard or transparent receiver".into(),
        ));
    }
    Ok(ufvk)
}

#[async_trait]
impl LedgerApp for OfficialApp {
    fn kind(&self) -> HwKind {
        HwKind::Official
    }

    async fn get_ufvk(&self, network: &Network, aindex: u32) -> Result<String> {
        let ledger = connect_ledger().await?;
        Ok(get_ufvk(&ledger, network, aindex).await?)
    }
}
