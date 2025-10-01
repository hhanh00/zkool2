use std::io::Write;

use anyhow::Result;
use byteorder::{WriteBytesExt, BE};
use hidapi::{self, HidApi, HidDevice};

pub fn open_ledger(api: &HidApi) -> Result<LedgerDevice> {
    for devinfo in api.device_list() {
        let vendor_id = devinfo.vendor_id();
        if vendor_id == 0x2C97 {
            // Ledger
            let device = devinfo.open_device(&api)?;
            return Ok(LedgerDevice { device, timeout: 10_000_000 });
        }
    }
    anyhow::bail!("No Ledger Device");
}

pub struct LedgerDevice {
    device: HidDevice,
    timeout: i32,
}

impl LedgerDevice {
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

        for (idx, chunk) in data.chunks(64 - 5).enumerate() {
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
                data.write_all(&buffer[7..size - 7])?;
            } else {
                data.write_all(&buffer[5..size - 5])?;
            }
            seqno += 1;
            if data.len() == data_len as usize {
                break;
            }
        }
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
    use hidapi::HidApi;

    #[test]
    fn t() -> anyhow::Result<()> {
        let api = HidApi::new()?;
        let _device = super::open_ledger(&api)?;
        Ok(())
    }
}
