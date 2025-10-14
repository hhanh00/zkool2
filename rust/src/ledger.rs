use std::sync::LazyLock;
use std::{io::Write, sync::Arc};

use byteorder::{WriteBytesExt, BE};
use hidapi::{self, HidApi, HidDevice};
use ledger_transport::Exchange;
#[cfg(target_os = "macos")]
use ledger_transport_zemu::TransportZemuHttp;
use tokio::runtime::Builder;
use tokio::sync::{mpsc, oneshot, Mutex};
use tonic::async_trait;

use crate::IntoAnyhow;

pub mod builder;
pub mod error;
pub mod fvk;
pub mod hashers;
pub mod legacy;

pub type LedgerError = error::Error;
pub type LedgerResult<T> = std::result::Result<T, LedgerError>;

pub fn open_ledger(api: &HidApi) -> LedgerResult<HidDevice> {
    for devinfo in api.device_list() {
        let vendor_id = devinfo.vendor_id();
        if vendor_id == 0x2C97 {
            // Ledger
            let device = devinfo.open_device(api)?;
            let _ = device.set_blocking_mode(true);
            return Ok(device);
        }
    }
    Err(LedgerError::NotFound)
}

#[derive(Clone)]
pub struct APDUCommand {
    pub cla: u8,
    pub ins: u8,
    pub p1: u8,
    pub p2: u8,
    pub data: Vec<u8>,
}

impl APDUCommand {
    pub fn to_bytes(&self) -> LedgerResult<Vec<u8>> {
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
    pub fn from_bytes(data: &[u8]) -> LedgerResult<Self> {
        let retcode = u16::from_be_bytes([data[data.len() - 2], data[data.len() - 1]]);
        let mut data2 = vec![];
        data2.extend_from_slice(&data[0..data.len() - 2]);
        Ok(Self {
            data: data2,
            retcode,
        })
    }
}

#[cfg(feature = "zemu")]
pub async fn connect_ledger() -> LedgerResult<LedgerDeviceZEMU> {
    let ledger = LEDGER_ZEMU.lock().await;
    Ok(ledger.clone().unwrap())
}

#[cfg(not(feature = "zemu"))]
pub async fn connect_ledger() -> LedgerResult<LedgerDevice> {
    {
        use std::ops::Deref;

        let ledger = LEDGER.lock().await;
        if let Some(ledger) = ledger.deref() {
            return Ok(ledger.clone());
        }
    };
    let mut ledger = LEDGER.lock().await;
    let device = LedgerDevice::new().await?;
    *ledger = Some(device.clone());
    Ok(device)
}

pub type ReponseChannel = oneshot::Sender<LedgerResult<APDUAnswer>>;

#[derive(Clone)]
pub struct LedgerDevice {
    tx: mpsc::Sender<(APDUCommand, ReponseChannel)>,
}

#[async_trait]
pub trait Device {
    async fn execute(&self, command: APDUCommand) -> LedgerResult<APDUAnswer>;
    async fn long_execute(
        &self,
        command: &APDUCommand,
        data: &[Vec<u8>],
    ) -> LedgerResult<APDUAnswer> {
        tracing::info!("Init Sending: {}", command.ins);
        let ins = command.ins;
        let c = APDUCommand {
            cla: command.cla,
            ins: command.ins,
            p1: 0,
            p2: command.p2,
            data: vec![],
        };
        let res = self.execute(c).await?;
        tracing::info!("res: {}", res.retcode);
        if res.retcode != 0x9000 {
            return Err(LedgerError::Execute(res.retcode, ins));
        }

        for p in data.iter() {
            for c in p.chunks(200) {
                tracing::info!("Sending: {}", hex::encode(c));
                let ins = command.ins;
                let command = APDUCommand {
                    cla: command.cla,
                    ins: command.ins,
                    p1: 1,
                    p2: command.p2,
                    data: c.to_vec(),
                };
                let res = self.execute(command).await?;
                tracing::info!("res: {}", res.retcode);
                if res.retcode != 0x9000 {
                    return Err(LedgerError::Execute(res.retcode, ins));
                }
            }
        }
        tracing::info!("Finish: {}", command.ins);
        let c = APDUCommand {
            cla: command.cla,
            ins: command.ins,
            p1: 2,
            p2: command.p2,
            data: vec![],
        };
        let res = self.execute(c).await?;
        tracing::info!("res: {}", res.retcode);
        Ok(res)
    }
}

#[async_trait]
impl Device for LedgerDevice {
    async fn execute(&self, command: APDUCommand) -> LedgerResult<APDUAnswer> {
        self.run(command).await
    }
}

impl LedgerDevice {
    pub async fn new() -> LedgerResult<Self> {
        let tx = Self::start().await?;
        Ok(LedgerDevice {
            tx,
        })
    }

    pub async fn start() -> LedgerResult<mpsc::Sender<(APDUCommand, ReponseChannel)>> {
        let hidapi = HidApi::new()?;
        let device = open_ledger(&hidapi)?;
        let (tx, mut rx) = mpsc::channel::<(APDUCommand, ReponseChannel)>(8);
        // spawn a single thread worker to make sure that access to the device is serialized
        std::thread::spawn(move || {
            let r = Builder::new_current_thread().enable_all().build().unwrap();
            r.block_on(async move {
                while let Some((command, sender)) = rx.recv().await {
                    let answer = async {
                        let c = command.to_bytes()?;
                        Self::write(&device, &c).await?;
                        let rep = Self::read(&device).await?;
                        let answer = APDUAnswer::from_bytes(&rep)?;
                        Ok::<_, LedgerError>(answer)
                    }.await;
                    let _ = sender.send(answer);
                }
            });
        });
        Ok(tx)
    }

    pub async fn run(&self, command: APDUCommand) -> LedgerResult<APDUAnswer> {
        let (tx, rx) = oneshot::channel::<LedgerResult<APDUAnswer>>();
        self.tx.send((command, tx)).await.anyhow()?;
        let rep = rx.await.anyhow()?;
        rep
    }

    async fn write(device: &HidDevice, data: &[u8]) -> LedgerResult<()> {
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

    async fn read(device: &HidDevice) -> LedgerResult<Vec<u8>> {
        let mut seqno = 0;
        let mut data_len = 0;
        let mut buffer = [0u8; 64];
        let mut data = vec![];

        loop {
            let size = device.read(&mut buffer)?;
            // the first chunk has the total length, therefore it must be larger
            if size < 5 || (seqno == 0 && size < 7) {
                return Err(LedgerError::Protocol("No header".into()));
            }
            if buffer[0] != 1 || buffer[1] != 1 || buffer[2] != 5 {
                return Err(LedgerError::Protocol("Invalid header".into()));
            }
            let this_seqno = u16::from_be_bytes([buffer[3], buffer[4]]);
            if this_seqno != seqno {
                return Err(LedgerError::Protocol("Invalid seqno".into()));
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

pub static LEDGER_ZEMU: LazyLock<tokio::sync::Mutex<Option<LedgerDeviceZEMU>>> =
    LazyLock::new(|| {
        #[cfg(target_os = "macos")]
        {
            let device = ledger_transport_zemu::TransportZemuHttp::new("192.168.18.13", 9999);
            let ledger = LedgerDeviceZEMU {
                device: Arc::new(device),
            };
            tokio::sync::Mutex::new(Some(ledger))
        }
        #[cfg(not(target_os = "macos"))]
        tokio::sync::Mutex::new(Some(LedgerDeviceZEMU {}))
    });

#[derive(Clone)]
pub struct LedgerDeviceZEMU {
    #[cfg(target_os = "macos")]
    pub device: Arc<TransportZemuHttp>,
}

#[async_trait]
impl Device for LedgerDeviceZEMU {
    #[cfg(target_os = "macos")]
    async fn execute(&self, command: APDUCommand) -> LedgerResult<APDUAnswer> {
        let res = self
            .device
            .exchange(&ledger_transport::APDUCommand {
                cla: command.cla,
                ins: command.ins,
                p1: command.p1,
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
    async fn execute(&self, command: APDUCommand) -> LedgerResult<APDUAnswer> {
        unimplemented!()
    }
}

pub static LEDGER: LazyLock<Mutex<Option<LedgerDevice>>> = LazyLock::new(|| Mutex::new(None));

#[cfg(test)]
mod tests;
