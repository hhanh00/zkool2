use std::io::{Cursor, Read, Write};
use std::sync::LazyLock;

use anyhow::Result;
use byteorder::{ReadBytesExt, LE};
use byteorder::{WriteBytesExt, BE};
use hidapi::{self, HidApi, HidDevice};
#[cfg(target_os = "macos")]
use ledger_transport_zemu::TransportZemuHttp;
use ledger_transport::Exchange;
use pczt::Pczt;
use sqlx::SqliteConnection;
use tokio::sync::Mutex;
use tonic::async_trait;
use zcash_keys::encoding::AddressCodec;
use zcash_primitives::legacy::TransparentAddress;
use zcash_primitives::transaction::Transaction;

use crate::coin::Network;
use crate::db::LEDGER_CODE;
use crate::Client;

pub mod builder;
pub mod hashers;

pub fn open_ledger(api: &HidApi) -> Result<LedgerDevice> {
    for devinfo in api.device_list() {
        let vendor_id = devinfo.vendor_id();
        if vendor_id == 0x2C97 {
            // Ledger
            let device = devinfo.open_device(api)?;
            let _ = device.set_blocking_mode(true);
            return Ok(LedgerDevice {
                device: Mutex::new(device),
                timeout: 100_000_000,
            });
        }
    }
    anyhow::bail!("No Ledger Device. Is it unlocked?");
}

pub async fn derive_hw_transparent_address(
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
    let rep = device.execute(&get_pk).await?;
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

pub fn get_trusted_input(tx: &Transaction, index: u32) -> Result<Vec<Vec<u8>>> {
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
    buffer.write_u32::<LE>(index)?;
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

pub async fn sign_transaction(
    network: &Network,
    _connection: &SqliteConnection,
    _account: u32,
    client: &mut Client,
    pczt: &Pczt,
) -> Result<()> {
    if !pczt.sapling().spends().is_empty()
        || !pczt.sapling().outputs().is_empty()
        || !pczt.orchard().actions().is_empty()
    {
        anyhow::bail!("Ledger only supports transparent transactions")
    }

    let ledger = LEDGER.lock().await;

    let tins = pczt.transparent().inputs();
    for tin in tins {
        let txid = tin.prevout_txid();
        let (_, prev_tx) = client.transaction(network, txid).await?;
        let apdu = get_trusted_input(&prev_tx, *tin.prevout_index())?;
        let r = ledger
            .long_execute(
                &APDUCommand {
                    cla: 0xE0,
                    ins: 0x42,
                    p1: 0,
                    p2: 0,
                    data: vec![],
                },
                &apdu,
            )
            .await?;
        if r.retcode != 0x9000 {
            anyhow::bail!("Ledger error {}", r.retcode)
        }
        let tin = r.data;
        println!("{}", hex::encode(&tin));
    }

    /*
    How to sign

    1. get trusted inputs
       1. send previous transactions to ins 0x42 and get the txid
    2. use 0x44
       1. send transaction up to outputs replacing tins by the trusted inputs
          1. (p1, p2) = (00, 05) then (80, 80)
       2. add prev pubscripts (should be our wallet addresses) + sequence
    3. use 0x4a
       1. set change output, p1 = FF
       2. set other outputs too
    4. send e04800000b 0000000000000100000000
    5. use 0x44 again but with 1 input at a time
    6. sign with 0x48 + path
    */

    todo!()
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

pub static LEDGER: LazyLock<Mutex<LedgerDevice>> = LazyLock::new(|| {
    let hidapi = HidApi::new().unwrap();
    let device = open_ledger(&hidapi).unwrap();
    Mutex::new(device)
});

pub struct LedgerDevice {
    device: Mutex<HidDevice>,
    timeout: i32,
}

#[async_trait]
pub trait Device {
    async fn execute(&self, command: &APDUCommand) -> Result<APDUAnswer>;
    async fn long_execute(&self, command: &APDUCommand, data: &[Vec<u8>]) -> Result<APDUAnswer>;
}

#[async_trait]
impl Device for LedgerDevice {
    async fn execute(&self, command: &APDUCommand) -> Result<APDUAnswer> {
        self.run(command).await
    }

    async fn long_execute(&self, command: &APDUCommand, data: &[Vec<u8>]) -> Result<APDUAnswer> {
        let c = APDUCommand {
            cla: command.cla,
            ins: command.ins,
            p1: 0,
            p2: command.p2,
            data: vec![],
        };
        let res = self.execute(&c).await?;
        if res.retcode != 0x9000 {
            anyhow::bail!("{}", res.retcode);
        }

        for p in data.iter() {
            for c in p.chunks(200) {
                let command = APDUCommand {
                    cla: command.cla,
                    ins: command.ins,
                    p1: 1,
                    p2: command.p2,
                    data: c.to_vec(),
                };
                let res = self.execute(&command).await?;
                if res.retcode != 0x9000 {
                    anyhow::bail!("{}", res.retcode);
                }
            }
        }
        let c = APDUCommand {
            cla: command.cla,
            ins: command.ins,
            p1: 2,
            p2: command.p2,
            data: vec![],
        };
        let res = self.execute(&c).await?;
        Ok(res)
    }
}

impl LedgerDevice {
    pub async fn run(&self, command: &APDUCommand) -> Result<APDUAnswer> {
        let c = command.to_bytes()?;
        self.write(&c).await?;
        let rep = self.read().await?;
        let answer = APDUAnswer::from_bytes(&rep)?;
        Ok(answer)
    }

    async fn write(&self, data: &[u8]) -> Result<()> {
        let device = self.device.lock().await;
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
            device.write(&buffer)?;
        }
        Ok(())
    }

    async fn read(&self) -> Result<Vec<u8>> {
        let device = self.device.lock().await;
        let mut seqno = 0;
        let mut data_len = 0;
        let mut buffer = [0u8; 64];
        let mut data = vec![];

        loop {
            let size = device.read_timeout(&mut buffer, self.timeout)?;
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

pub static LEDGER_ZEMU: LazyLock<tokio::sync::Mutex<LedgerDeviceZEMU>> = LazyLock::new(|| {
    #[cfg(target_os = "macos")]
    {
        let device = ledger_transport_zemu::TransportZemuHttp::new("192.168.18.13", 9999);
        let ledger = LedgerDeviceZEMU { device };
        tokio::sync::Mutex::new(ledger)
    }
    #[cfg(not(target_os = "macos"))]
    tokio::sync::Mutex::new(LedgerDeviceZEMU {})
});

    pub struct LedgerDeviceZEMU {
        #[cfg(target_os = "macos")]
        pub device: TransportZemuHttp,
    }

    #[async_trait]
    impl Device for LedgerDeviceZEMU {
        #[cfg(target_os = "macos")]
        async fn execute(&self, command: &APDUCommand) -> Result<APDUAnswer> {
            let res = self
                .device
                .exchange(&ledger_transport::APDUCommand {
                    cla: command.cla,
                    ins: command.ins,
                    p1: 0,
                    p2: command.p2,
                    data: command.data.clone(),
                })
                .await?;
            Ok(APDUAnswer {
                data: res.data().to_vec(),
                retcode: res.retcode(),
            })
        }

        #[cfg(not(target_os = "macos"))]
        async fn execute(&self, command: &APDUCommand) -> Result<APDUAnswer> {
            unimplemented!()
        }

        #[cfg(target_os = "macos")]
        async fn long_execute(
            &self,
            command: &APDUCommand,
            data: &[Vec<u8>],
        ) -> anyhow::Result<APDUAnswer> {
            self.device
                .exchange(&ledger_transport::APDUCommand {
                    cla: command.cla,
                    ins: command.ins,
                    p1: 0,
                    p2: command.p2,
                    data: vec![],
                })
                .await?;

            for d in data.iter() {
                for c in d.chunks(200) {
                    let res = self
                        .device
                        .exchange(&ledger_transport::APDUCommand {
                            cla: command.cla,
                            ins: command.ins,
                            p1: 1,
                            p2: command.p2,
                            data: c,
                        })
                        .await?;
                    assert_eq!(res.retcode(), 0x9000);
                }
            }

            let res = self
                .device
                .exchange(&ledger_transport::APDUCommand {
                    cla: command.cla,
                    ins: command.ins,
                    p1: 2,
                    p2: command.p2,
                    data: vec![],
                })
                .await?;
            Ok(APDUAnswer {
                data: res.data().to_vec(),
                retcode: res.retcode(),
            })
        }

        #[cfg(not(target_os = "macos"))]
        async fn long_execute(
            &self,
            command: &APDUCommand,
            data: &[Vec<u8>],
        ) -> anyhow::Result<APDUAnswer> {
            unimplemented!()
        }
    }

#[cfg(test)]
mod tests {
    use bech32::{Bech32m, Hrp};
    use pczt::roles::{
        low_level_signer::Signer, spend_finalizer::SpendFinalizer,
        tx_extractor::TransactionExtractor,
    };
    use secp256k1::{ecdsa::Signature, PublicKey};
    use sqlx::{Acquire, SqlitePool};
    use std::{
        fs::File,
        io::{BufReader, Cursor, Read},
    };
    use zcash_primitives::legacy::Script;

    use byteorder::ReadBytesExt;
    use sapling_crypto::{keys::FullViewingKey, Diversifier, PaymentAddress};
    use zcash_address::unified::{self, Encoding, Ufvk};
    use zcash_protocol::consensus::MainNetwork;

    use crate::{
        api::pay::PcztPackage,
        coin::{Coin, Network, ServerType},
        ledger::{
            hashers::{
                header_hasher, orchard_hasher, output_hasher, prevout_hasher, sapling_hasher,
                sequence_hasher, sig_hasher, spend_hasher, transparent_hasher, zoutput_hasher,
            },
            APDUCommand
        },
    };

    use super::*;

    #[tokio::test]
    async fn test_get_trusted_inputs() -> anyhow::Result<()> {
        let mut txid =
            hex::decode("339df469e5b5e0259e214a1c653cdcb28c2c2e9664c5b5df5d518974950c36e1")?;
        txid.reverse();
        let coin = Coin::new(ServerType::Lwd, "https://zec.rocks", false, "test.db", None).await?;
        let mut client = coin.client().await?;
        let (_, tx) = client.transaction(&Network::Main, &txid).await?;
        let payload = super::get_trusted_input(&tx, 0)?;

        LEDGER_ZEMU
            .lock()
            .await
            .long_execute(
                &APDUCommand {
                    cla: 0xE0,
                    ins: 0x42,
                    p1: 0,
                    p2: 0,
                    data: vec![],
                },
                &payload,
            )
            .await?;
        Ok(())
    }

    // This is for the Transparent Only Ledger app
    #[allow(dead_code)]
    async fn get_pk() -> anyhow::Result<()> {
        let get_pk = APDUCommand {
            cla: 0xE0,
            ins: 0x40,
            p1: 0,
            p2: 0,
            data: vec![],
        };
        let rep = LEDGER_ZEMU
            .lock()
            .await
            .long_execute(
                &get_pk,
                &[hex::decode("058000002C80000085800000000000000000000000").unwrap()],
            )
            .await?;
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

    #[tokio::test]
    pub async fn get_device_info() -> Result<()> {
        let ledger = LEDGER.lock().await;
        let res = ledger
            .long_execute(
                &APDUCommand {
                    cla: 0x85,
                    ins: 0x00,
                    p1: 0,
                    p2: 0,
                    data: vec![],
                },
                &[vec![]],
            )
            .await?;
        assert_eq!(res.retcode, 0x9000);
        println!("{}", hex::encode(&res.data));
        Ok(())
    }

    #[tokio::test]
    pub async fn get_taddress() -> Result<()> {
        let get_address = APDUCommand {
            cla: 0x85,
            ins: 0x01,
            p1: 0,
            p2: 0,
            data: vec![],
        };
        let mut data = vec![];
        data.write_u32::<LE>(44 | 0x80000000)?;
        data.write_u32::<LE>(133 | 0x80000000)?;
        data.write_u32::<LE>(0x80000000)?;
        data.write_u32::<LE>(0)?;
        data.write_u32::<LE>(0)?;
        let ledger = LEDGER_ZEMU.lock().await;
        let res = ledger.long_execute(&get_address, &[data]).await?;
        assert_eq!(res.retcode, 0x9000);
        let pk = &res.data[0..33];
        println!("{}", hex::encode(pk));
        let address = &res.data[33..];
        let pk = bech32::encode::<Bech32m>(Hrp::parse_unchecked("zpk"), pk)?;
        println!("{pk}");
        let address = String::from_utf8(address.to_vec())?;
        println!("{address}");
        assert_eq!(address, "t1h31WzbruQhnwHg4XDJ5anLM7CAtwjXmPt");
        Ok(())
    }

    #[tokio::test]
    pub async fn get_fvk() -> Result<()> {
        // this test will fail with "Inner Ledger error" the first time it runs
        // because user needs to confirm on the device
        // Run it once, then go to the web ui and confirm the operation
        // Run it again and it should pass
        let ledger = LEDGER.lock().await;
        let res = ledger
            .execute(&APDUCommand {
                cla: 0x85,
                ins: 0xF3,
                p1: 1,
                p2: 0,
                data: 0x80000000u32.to_le_bytes().to_vec(),
            })
            .await?;
        assert_eq!(res.retcode, 0x9000);
        let fvk = hex::encode(&res.data);
        println!("{fvk}");
        assert_eq!(fvk, "d17091f057e2d641328172642f06f821893a564ec8ab98fdd4ca462b8791de5c788c96b31e5e476e954c1a18bd4f1278358924ec9a22d096fe3954d815e353605940cfcf8388fb5e54ebc6f1c9f75a5eddf35227e3d1c4ef003e6f64cd7672db");
        Ok(())
    }

    #[tokio::test]
    pub async fn get_address() -> Result<()> {
        let ledger = LEDGER_ZEMU.lock().await;
        let res = ledger
            .execute(&APDUCommand {
                cla: 0x85,
                ins: 0x11,
                p1: 0,
                p2: 0,
                data: 0x80000000u32.to_le_bytes().to_vec(),
            })
            .await?;
        assert_eq!(res.retcode, 0x9000);
        let address = hex::encode(&res.data);
        println!("{address}");
        assert_eq!(address, "a7b6aa86c0c01e5cb4c2285e1d5226c4121687100e3ada3ad60c420516ecc7aeae321e74f1db380bfea40b7a733135376d3234706b71637130396564787a397030703635337863736670647063737063616435776b6b70337071323968766337683275767337776e63616b7771746c366a716b786e39333970");
        Ok(())
    }

    #[test]
    fn payment_address() -> Result<()> {
        let address_hex = hex::decode("a7b6aa86c0c01e5cb4c2285e1d5226c4121687100e3ada3ad60c420516ecc7aeae321e74f1db380bfea40b7a733135376d3234706b71637130396564787a397030703635337863736670647063737063616435776b6b70337071323968766337683275767337776e63616b7771746c366a716b786e39333970")?;
        let address = &address_hex[0..43];
        let div = hex::encode(&address_hex[0..11]);
        println!("{div}");
        let pa = PaymentAddress::from_bytes(&address.try_into().unwrap()).unwrap();
        println!("{}", pa.encode(&MainNetwork));
        let address = String::from_utf8(address_hex[43..].to_vec())?;
        println!("{address}");
        assert_eq!(
            address,
            "zs157m24pkqcq09edxz9p0p653xcsfpdpcspcad5wkkp3pq29hvc7h2uvs7wncakwqtl6jqkxn939p"
        );
        Ok(())
    }

    #[test]
    pub fn ufvk() -> Result<()> {
        let fvk = hex::decode("de514bb8eba2793731926578513d8ea724d1e4b21fcf8a53b7236711a27ba7bf05eda7736c88143790f66a1793f117100b8b7d0c60c115ee7a2d0e189c4fb416a5077c6d42e7c18de0353751b361a55e90fccbbef3d12fafba1d43a4367feefa")?;
        let sapfvk = FullViewingKey::read(&*fvk)?;
        let div: [u8; 11] = hex::decode("a7b6aa86c0c01e5cb4c228")?.try_into().unwrap();
        let pa = sapfvk.vk.to_payment_address(Diversifier(div)).unwrap();
        println!("{}", pa.encode(&MainNetwork));
        let mut dk = [42u8; 128]; // arbitrary dk because we don't know the real one from the Ledger
        dk[0..96].clone_from_slice(&fvk);
        let sfvk = unified::Fvk::Sapling(dk);
        let ufvk = Ufvk::try_from_items(vec![sfvk])?;
        let ufvk = ufvk.encode(&zcash_protocol::consensus::NetworkType::Main);
        println!("{ufvk}");
        assert_eq!(ufvk, "uview1hytkw2afs80j0zvj3w0nutqs58rpf2qvuygw47k3qmjqj8w9vlwxjp00dpk8tfp6e5jdq0zavetsu5jugxpsqqwssjeh9lxsnugenctuyjhf6my639pv7agspcsvmgk5upj2zjkwm3u98h807sdj5dkvtrle5x2uajl6gzj4ryhuz0sfm2j3g95hm6az2an4tknu0yecmefsrrwqxv6fxgwqpf44awj6fnrhxlytcut20faw");
        Ok(())
    }

    #[tokio::test]
    pub async fn sign_transparent() -> Result<()> {
        let stage = 2;
        let network = Network::Main;
        let pool = SqlitePool::connect("ledger.db").await?;
        let mut connection = pool.acquire().await?;
        let connection = connection.acquire().await?;
        let mut db_tx = connection.begin().await?;
        let account = 1;
        let s_account = 2;

        let file = File::open("t2s.bin")?;
        let package = bincode::decode_from_reader::<PcztPackage, _, _>(
            BufReader::new(file),
            bincode::config::legacy(),
        )?;
        let pczt = Pczt::parse(&package.pczt).unwrap();
        let (pk, address): (Vec<u8>, String) = sqlx::query_as(
            "SELECT pk, address FROM transparent_address_accounts WHERE account = ?1",
        )
        .bind(account)
        .fetch_one(&mut *db_tx)
        .await?;
        let pk = PublicKey::from_slice(&pk)?;
        let address = TransparentAddress::decode(&network, &address)?;

        let (xvk,): (Vec<u8>,) =
            sqlx::query_as("SELECT xvk FROM sapling_accounts WHERE account = ?1")
                .bind(s_account)
                .fetch_one(&mut *db_tx)
                .await?;
        let xvk = FullViewingKey::read(&*xvk)?;
        let ovk = xvk.ovk;

        let mut buffers = vec![];
        buffers.push(vec![]);

        let mut data = vec![];
        data.write_u8(pczt.transparent().inputs().len() as u8)?;
        data.write_u8(pczt.transparent().outputs().len() as u8)?;
        data.write_u8(pczt.sapling().spends().len() as u8)?;
        data.write_u8(pczt.sapling().outputs().len() as u8)?;
        buffers.push(data);

        println!(
            "{} {}",
            pczt.transparent().inputs().len(),
            pczt.transparent().outputs().len()
        );
        println!(
            "{} {}",
            pczt.sapling().spends().len(),
            pczt.sapling().outputs().len()
        );

        let tbundle = pczt.transparent();
        for tin in tbundle.inputs() {
            let mut data = vec![];
            data.write_u32::<LE>(44 + 0x80000000)?;
            data.write_u32::<LE>(133 + 0x80000000)?;
            data.write_u32::<LE>(0x80000000)?;
            data.write_u32::<LE>(0)?; // scope
            data.write_u32::<LE>(0)?; // dindex
            address.script().write(&mut data)?;
            data.write_u64::<LE>(*tin.value())?;
            assert_eq!(data.len(), 54);
            buffers.push(data);
        }

        for tout in tbundle.outputs() {
            let mut data = vec![];
            Script(tout.script_pubkey().to_vec()).write(&mut data)?;
            data.write_u64::<LE>(*tout.value())?;
            assert_eq!(data.len(), 34);
            buffers.push(data);
        }

        let sbundle = pczt.sapling();
        for sout in sbundle.outputs() {
            let mut data = vec![];
            let recipient = sout.recipient().expect("Must have a recipient");
            data.write_all(&recipient)?;
            data.write_u64::<LE>(sout.value().expect("Must have value"))?;
            data.write_u8(0xF6)?; // Memo type
            data.write_u8(0x01)?; // Have OVK
            data.write_all(&ovk.0)?;
            assert_eq!(data.len(), 85);
            buffers.push(data);
        }

        let init_tx = APDUCommand {
            cla: 0x85,
            ins: 0xA0,
            p1: 0,
            p2: 5,
            data: vec![],
        };

        let ledger = LEDGER_ZEMU.lock().await;
        if stage == 1 {
            let total_len = buffers.iter().map(|b| b.len()).sum::<usize>();
            assert_eq!(
                total_len,
                4 + 54 * tbundle.inputs().len()
                    + 34 * tbundle.outputs().len()
                    + 85 * sbundle.outputs().len()
            );
            let res = ledger.long_execute(&init_tx, &buffers).await?;
            assert_eq!(res.retcode, 0x9000);
        }

        let mut buffers = vec![];
        buffers.push(vec![]);
        for tin in tbundle.inputs() {
            let mut data = vec![];
            data.write_all(tin.prevout_txid())?;
            data.write_u32::<LE>(*tin.prevout_index())?;
            data.write_u8(0x19)?;
            data.write_all(tin.script_pubkey())?;
            data.write_u64::<LE>(*tin.value())?;
            data.write_u32::<LE>(tin.sequence().unwrap_or(0xFFFFFFFFu32))?;
            assert_eq!(data.len(), 74);
            buffers.push(data);
        }
        /* hashes
           header:
               version, group, consensus, locktime, expiry: 5*4 = 20
           transparent:
               prevout, sequence, output: 3*32 = 96
           sapling:
               spends, outputs, net: 2*32 + 8 = 72
           orchard: 32
           = 220
        */
        let mut sighashes = vec![];
        let header = pczt.global();
        let expiration = header.expiry_height();
        let version = header.tx_version() | 0x80000000;
        let version_group = header.version_group_id();
        let branch = header.consensus_branch_id();
        sighashes.write_u32::<LE>(version)?;
        sighashes.write_u32::<LE>(*version_group)?;
        sighashes.write_u32::<LE>(*branch)?;
        sighashes.write_u32::<LE>(0)?;
        sighashes.write_u32::<LE>(*expiration)?;

        println!("H: {}", hex::encode(header_hasher(&pczt)?));
        println!("T: {}", hex::encode(transparent_hasher(&pczt)?));
        println!("S: {}", hex::encode(sapling_hasher(&pczt)?));
        println!("O: {}", hex::encode(orchard_hasher(&pczt)?));
        println!("Sig: {}", hex::encode(sig_hasher(&pczt)?));

        sighashes.write_all(&prevout_hasher(&pczt)?)?;
        sighashes.write_all(&sequence_hasher(&pczt)?)?;
        sighashes.write_all(&output_hasher(&pczt)?)?;

        sighashes.write_all(&spend_hasher(&pczt)?)?;
        sighashes.write_all(&zoutput_hasher(&pczt)?)?;
        sighashes.write_i64::<LE>(0)?;

        sighashes.write_all(&orchard_hasher(&pczt)?)?;
        buffers.push(sighashes);

        let check_sign = APDUCommand {
            cla: 0x85,
            ins: 0xA3,
            p1: 0,
            p2: 5,
            data: vec![],
        };

        if stage == 3 {
            let res = ledger.long_execute(&check_sign, &buffers).await?;
            assert_eq!(res.retcode, 0x9000);
            println!(">> {}", hex::encode(&res.data));
        }

        let mut signatures = vec![];
        for _ in tbundle.inputs() {
            let get_signature = APDUCommand {
                cla: 0x85,
                ins: 0xA5,
                p1: 0,
                p2: 0,
                data: vec![],
            };
            if stage == 3 {
                let res = ledger.long_execute(&get_signature, &[vec![]]).await?;
                assert_eq!(res.retcode, 0x9000);
                let signature = res.data[..64].to_vec();
                let signature = Signature::from_compact(&signature)?;
                signatures.push(signature);
            }
        }

        let sig_hex = "b154b87733d9040a995880a54e3575b0169920775c080eb71f4c2b9143ca4c454085a5665f997cc8d2652560ee9f775a705500a5236c25f989c70d213ed6b7de";
        if stage == 3 {
            let sig = hex::encode(signatures[0].serialize_compact());
            assert_eq!(sig, sig_hex);
        }

        if stage == 4 {
            let signature = Signature::from_compact(&hex::decode(sig_hex).unwrap())?;
            let signer = Signer::new(pczt.clone());
            let signer = signer
                .sign_transparent_with(|_pczt, tbundle, _| {
                    tbundle.inputs_mut()[0].apply_signature(&pk, &signature);
                    Ok::<_, zcash_transparent::pczt::ParseError>(())
                })
                .unwrap();
            let pczt = signer.finish();
            let pczt = SpendFinalizer::new(pczt).finalize_spends().unwrap();

            let tx_extractor = TransactionExtractor::new(pczt);
            let tx = tx_extractor.extract().unwrap();
            println!("{}", tx.txid());
            let mut tx_bytes = vec![];
            tx.write(&mut tx_bytes).unwrap();
            println!("{}", hex::encode(&tx_bytes));

            let coin =
                Coin::new(ServerType::Lwd, "https://zec.rocks", false, "test.db", None).await?;
            let mut client = coin.client().await?;
            let rep = client.post_transaction(*expiration, &tx_bytes).await?;
            println!("{rep}");
        }
        Ok(())
    }

    #[tokio::test]
    pub async fn sign_tx() -> Result<()> {
        let file = File::open("ledger.bin")?;
        let package = bincode::decode_from_reader::<PcztPackage, _, _>(
            BufReader::new(file),
            bincode::config::legacy(),
        )?;
        let pczt = Pczt::parse(&package.pczt).unwrap();

        APDUCommand {
            cla: 0x85,
            ins: 0xA0,
            p1: 0,
            p2: 0,
            data: vec![],
        };

        let mut data = vec![];
        data.write_u8(pczt.transparent().inputs().len() as u8)?;
        data.write_u8(pczt.transparent().outputs().len() as u8)?;
        data.write_u8(pczt.sapling().spends().len() as u8)?;
        data.write_u8(pczt.sapling().outputs().len() as u8)?;

        assert!(pczt.sapling().spends().is_empty());
        assert!(pczt.sapling().outputs().is_empty());

        Ok(())
    }
}
