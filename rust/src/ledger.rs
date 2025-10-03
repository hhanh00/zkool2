use std::io::{Cursor, Read, Write};

use anyhow::Result;
use byteorder::{ReadBytesExt, LE};
use byteorder::{WriteBytesExt, BE};
use hidapi::{self, HidApi, HidDevice};
use zcash_keys::encoding::AddressCodec;
use zcash_primitives::legacy::TransparentAddress;
use zcash_primitives::transaction::Transaction;

use crate::coin::Network;
use crate::db::LEDGER_CODE;

pub fn open_ledger(api: &HidApi) -> Result<LedgerDevice> {
    for devinfo in api.device_list() {
        let vendor_id = devinfo.vendor_id();
        if vendor_id == 0x2C97 { // LEDGER
            // Ledger
            let device = devinfo.open_device(api)?;
            let _ = device.set_blocking_mode(true);
            return Ok(LedgerDevice {
                device,
                timeout: 100_000_000,
            });
        }
    }
    anyhow::bail!("No Ledger Device. Is it unlocked?");
}

pub fn derive_hw_transparent_address(
    network: &Network,
    hw_code: u32,
    aindex: u32,
    scope: u32,
    dindex: u32,
) -> Result<(Vec<u8>, TransparentAddress)> {
    assert_eq!(hw_code, LEDGER_CODE);
    let api = HidApi::new()?;
    let device = open_ledger(&api)?;

    let mut params = vec![];
    // 5 parts, begins with m/44'/133'
    params.extend_from_slice(&hex::decode("058000002C80000085").unwrap());
    // account'
    params.extend_from_slice(&(aindex | 0x80000000).to_be_bytes());
    // external
    params.extend_from_slice(&(scope).to_be_bytes());
    // dindex
    params.extend_from_slice(&(dindex).to_be_bytes());

    let get_pk = APDUCommand {
        cla: 0xE0,
        ins: 0x40,
        p1: 0,
        p2: 0,
        data: params,
    };
    let rep = device.execute(&get_pk)?;
    if rep.retcode != 0x9000 {
        anyhow::bail!("Ledger error {}", rep.retcode);
    }
    let mut data = Cursor::new(&rep.data);
    let pk_len = data.read_u8()? as usize;
    let mut pk = vec![0u8; pk_len];
    data.read_exact(&mut pk)?;
    let address_len = data.read_u8()? as usize;
    let mut address = vec![0u8; address_len];
    data.read_exact(&mut address)?;
    let address = String::from_utf8(address)?;
    let address = TransparentAddress::decode(network, &address)?;
    Ok((pk, address))
}

pub fn get_trusted_input(tx: &Transaction, index: usize) -> Result<Vec<Vec<u8>>> {
    /*
    - output index - 4 bytes
    - version
    - version group
    - branch id
    - # tin
        - prev txid
        - prev vout
        - redeem script
        - sequence
    - # tout
        - value
        - pubscript
    - # sin/sout/oact
    - if sapling
        - value balance
        - anchor if spend
        - spends
            - cv
            - nf
            - rk
        - outputs
            - cmu
            - epk
            - cenc comp
        - outputs
            - memo
        - outputs
            - cv
            - cenc authd
            - cout
    - if orchard
        - for each action
            - nf
            - cmx
            - epk
            - cenc comp
        - for each action
            - memos
        - for each action
            - cv
            - rk
            - authd
            - cout
        - flags
        - balance
        - anchor
    */

    let mut buffers = vec![];
    let mut buffer = vec![];
    buffer.write_u32::<LE>(index as u32)?;
    buffer.write_u32::<LE>(tx.version().header())?;
    buffer.write_u32::<LE>(tx.version().version_group_id())?;
    buffer.write_u32::<LE>(tx.consensus_branch_id().into())?;
    if let Some(tbundle) = tx.transparent_bundle() {
        buffer.write_u8(tbundle.vin.len() as u8)?; // TODO use compact
        for tin in tbundle.vin.iter() {
            buffer.write_all(tin.prevout().hash())?;
            buffer.write_u32::<LE>(tin.prevout().n())?;
            tin.script_sig().write(&mut buffer)?;
            buffer.write_u32::<LE>(tin.sequence())?;
            buffers.push(std::mem::take(&mut buffer));
            buffer.clear();
        }
        buffer.write_u8(tbundle.vout.len() as u8)?; // TODO use compact
        for tout in tbundle.vout.iter() {
            buffer.write_u64::<LE>(tout.value().into_u64())?;
            tout.script_pubkey().write(&mut buffer)?;
            buffers.push(std::mem::take(&mut buffer));
            buffer.clear();
        }
    } else {
        buffer.write_u16::<LE>(0)?;
        buffers.push(std::mem::take(&mut buffer));
        buffer.clear();
    }
    let (sin, sout) = tx
        .sapling_bundle()
        .map(|b| (b.shielded_spends().len(), b.shielded_outputs().len()))
        .unwrap_or_default();
    let oact = tx
        .orchard_bundle()
        .map(|b| b.actions().len())
        .unwrap_or_default();
    buffer.write_u8(sin as u8)?; // TODO use compact
    buffer.write_u8(sout as u8)?; // TODO use compact
    buffer.write_u8(oact as u8)?; // TODO use compact
    buffers.push(std::mem::take(&mut buffer));
    buffer.clear();

    if let Some(sbundle) = tx.sapling_bundle() {
        buffer.write_i64::<LE>(sbundle.value_balance().into())?;
        if sin > 0 {
            buffer.write_all(&sbundle.shielded_spends()[0].anchor().to_bytes())?;
        }
        buffers.push(std::mem::take(&mut buffer));
        buffer.clear();
        for sin in sbundle.shielded_spends().iter() {
            buffer.write_all(&sin.cv().to_bytes())?;
            buffer.write_all(&sin.nullifier().to_vec())?;
            let rk: [u8; 32] = (*sin.rk()).into();
            buffer.write_all(&rk)?;
            buffers.push(std::mem::take(&mut buffer));
            buffer.clear();
        }
        for sout in sbundle.shielded_outputs() {
            buffer.write_all(&sout.cmu().to_bytes())?;
            buffer.write_all(&sout.ephemeral_key().0)?;
            buffer.write_all(&sout.enc_ciphertext()[0..52])?;
            buffers.push(std::mem::take(&mut buffer));
            buffer.clear();
        }
        for sout in sbundle.shielded_outputs() {
            for i in 0..4 {
                buffer.write_all(&sout.enc_ciphertext()[52 + i * 128..52 + (i + 1) * 128])?;
                buffers.push(std::mem::take(&mut buffer));
                buffer.clear();
            }
        }
        for sout in sbundle.shielded_outputs() {
            buffer.write_all(&sout.cv().to_bytes())?;
            buffer.write_all(&sout.enc_ciphertext()[52 + 512..])?;
            buffer.write_all(sout.out_ciphertext())?;
            buffers.push(std::mem::take(&mut buffer));
            buffer.clear();
        }
    }

    if let Some(obundle) = tx.orchard_bundle() {
        for a in obundle.actions().iter() {
            buffer.write_all(&a.nullifier().to_bytes())?;
            buffer.write_all(&a.cmx().to_bytes())?;
            buffer.write_all(&a.encrypted_note().epk_bytes)?;
            buffer.write_all(&a.encrypted_note().enc_ciphertext[..52])?;
            buffers.push(std::mem::take(&mut buffer));
            buffer.clear();
        }
        for a in obundle.actions().iter() {
            for i in 0..4 {
                buffer.write_all(
                    &a.encrypted_note().enc_ciphertext[52 + i * 128..52 + (i + 1) * 128],
                )?;
                buffers.push(std::mem::take(&mut buffer));
                buffer.clear();
            }
        }
        for a in obundle.actions().iter() {
            buffer.write_all(&a.cv_net().to_bytes())?;
            let rk: [u8; 32] = (a.rk()).into();
            buffer.write_all(&rk)?;
            buffer.write_all(&a.encrypted_note().enc_ciphertext[52 + 512..])?;
            buffer.write_all(&a.encrypted_note().out_ciphertext)?;
            buffers.push(std::mem::take(&mut buffer));
            buffer.clear();
        }

        buffer.write_u8(obundle.flags().to_byte())?;
        buffer.write_i64::<LE>(obundle.value_balance().into())?;
        buffer.write_all(&obundle.anchor().to_bytes())?;
        buffers.push(std::mem::take(&mut buffer));
        buffer.clear();
    }
    buffer.write_u32::<LE>(tx.lock_time())?;
    buffer.write_u8(4)?; // len of extra data
    buffer.write_u32::<LE>(tx.expiry_height().into())?;
    buffers.push(std::mem::take(&mut buffer));
    buffer.clear();

    Ok(buffers)
}

pub fn sign_transaction() -> Result<()> {
    todo!()
}

pub struct LedgerDevice {
    device: HidDevice,
    timeout: i32,
}

impl LedgerDevice {
    pub fn long_execute(&self, command: &APDUCommand, data: &[Vec<u8>]) -> Result<APDUAnswer> {
        for (i, p) in data.iter().enumerate() {
            let c = APDUCommand {
                cla: command.cla,
                ins: command.ins,
                p1: command.p1 | (if i != 0 { 0x80 } else { 0x00 }),
                p2: command.p2,
                data: p.clone(),
            };
            let res = self.execute(&c)?;
            if res.retcode != 0x9000 {
                anyhow::bail!("{}", res.retcode);
            }

            if i == data.len() - 1 {
                return Ok(res);
            }
        }
        anyhow::bail!("No payload");
    }

    pub fn execute(&self, command: &APDUCommand) -> Result<APDUAnswer> {
        let c = command.to_bytes()?;
        self.write(&c)?;
        let rep = self.read()?;
        let answer = APDUAnswer::from_bytes(&rep)?;
        Ok(answer)
    }

    fn write(&self, data: &[u8]) -> Result<()> {
        // data is prefixed by its length
        let mut prefixed_data = Vec::<u8>::with_capacity(data.len() + 2);
        prefixed_data.write_u16::<BE>(data.len() as u16)?;
        prefixed_data.write_all(data)?;

        // it is split into chunks of 64 bytes
        // the first 5 bytes are the header
        // channel (0x0101), tag (0x05), seqno (u16)

        // we have an extra 1 byte when we write
        // it is not there when we read
        let mut buffer = [0u8; 65]; // 1 byte prefix + 64 byte buffer
        buffer[1] = 1; // channel
        buffer[2] = 1; // channel
        buffer[3] = 5; // tag

        for (idx, chunk) in prefixed_data.chunks(64 - 5).enumerate() {
            let seqno = idx as u16;
            buffer[4..6].copy_from_slice(&seqno.to_be_bytes());
            buffer[6..6 + chunk.len()].copy_from_slice(chunk);
            self.device.write(&buffer)?;
        }
        Ok(())
    }

    fn read(&self) -> Result<Vec<u8>> {
        let mut seqno = 0;
        let mut data_len = 0;
        let mut buffer = [0u8; 64];
        let mut data = vec![];

        loop {
            let size = self.device.read_timeout(&mut buffer, self.timeout)?;
            // the first chunk has the total length, therefore it must be larger
            if size < 5 || (seqno == 0 && size < 7) {
                anyhow::bail!("No header");
            }
            if buffer[0] != 1 || buffer[1] != 1 || buffer[2] != 5 {
                anyhow::bail!("Invalid header");
            }
            let this_seqno = u16::from_be_bytes([buffer[3], buffer[4]]);
            if this_seqno != seqno {
                anyhow::bail!("Invalid seqno");
            }
            if seqno == 0 {
                data_len = u16::from_be_bytes([buffer[5], buffer[6]]);
                data.write_all(&buffer[7..size])?;
            } else {
                data.write_all(&buffer[5..size])?;
            }
            seqno += 1;
            if data.len() >= data_len as usize {
                break;
            }
        }
        data.truncate(data_len as usize);
        Ok(data)
    }
}

pub struct APDUCommand {
    pub cla: u8,
    pub ins: u8,
    pub p1: u8,
    pub p2: u8,
    pub data: Vec<u8>,
}

impl APDUCommand {
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buffer = vec![];
        buffer.write_u8(self.cla)?;
        buffer.write_u8(self.ins)?;
        buffer.write_u8(self.p1)?;
        buffer.write_u8(self.p2)?;
        buffer.write_u8(self.data.len() as u8)?;
        buffer.write_all(&self.data)?;
        Ok(buffer)
    }
}

pub struct APDUAnswer {
    pub data: Vec<u8>,
    pub retcode: u16,
}

impl APDUAnswer {
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let retcode = u16::from_be_bytes([data[data.len() - 2], data[data.len() - 1]]);
        let mut data2 = vec![];
        data2.extend_from_slice(&data[0..data.len() - 2]);
        Ok(Self {
            data: data2,
            retcode,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{io::{Cursor, Read}, sync::{LazyLock, Mutex}};

    use byteorder::ReadBytesExt;
    use hidapi::HidApi;

    use crate::{
        coin::{Coin, Network, ServerType},
        ledger::{APDUCommand, LedgerDevice},
    };

    pub static LEDGER: LazyLock<Mutex<LedgerDevice>> = LazyLock::new(|| {
        let hidapi = HidApi::new().unwrap();
        let device = super::open_ledger(&hidapi).unwrap();
        Mutex::new(device)
    });

    #[tokio::test]
    async fn test_get_trusted_inputs() -> anyhow::Result<()> {
        let mut txid =
            hex::decode("339df469e5b5e0259e214a1c653cdcb28c2c2e9664c5b5df5d518974950c36e1")?;
        txid.reverse();
        let coin = Coin::new(ServerType::Lwd, "https://zec.rocks", false, "test.db", None).await?;
        let mut client = coin.client().await?;
        let (_, tx) = client.transaction(&Network::Main, &txid).await?;
        let payload = super::get_trusted_input(&tx, 0)?;

        LEDGER.lock().unwrap().long_execute(
            &APDUCommand {
                cla: 0xE0,
                ins: 0x42,
                p1: 0,
                p2: 0,
                data: vec![],
            },
            &payload,
        )?;
        Ok(())
    }

    #[test]
    fn get_pk() -> anyhow::Result<()> {
        let get_pk = APDUCommand {
            cla: 0xE0,
            ins: 0x40,
            p1: 0,
            p2: 0,
            data: hex::decode("058000002C80000085800000000000000000000000").unwrap(),
        };
        let rep = LEDGER.lock().unwrap().execute(&get_pk)?;
        println!("{}", rep.retcode);
        let mut data = Cursor::new(&rep.data);
        let pk_len = data.read_u8()? as usize;
        let mut pk = vec![0u8; pk_len];
        data.read_exact(&mut pk)?;
        let address_len = data.read_u8()? as usize;
        let mut address = vec![0u8; address_len];
        data.read_exact(&mut address)?;
        let address = String::from_utf8(address)?;
        println!("{address}");
        Ok(())
    }
}
