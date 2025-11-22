use std::{fs::File, io::Read, sync::{LazyLock, Mutex}};

use anyhow::Result;
use flutter_rust_bridge::frb;
use qrcode::{bits::Bits, EcLevel};
use raptorq::{Decoder, Encoder, EncodingPacket, ObjectTransmissionInformation};

pub struct RaptorQParams {
    pub version: u16,
    pub ec_level: u8,
    pub repair: u32,
}

#[frb]
pub async fn encode(path: &str, params: RaptorQParams) -> Result<Vec<Vec<u8>>> {
    let mut file = File::open(path)?;
    let mut data = vec![];
    file.read_to_end(&mut data)?;
    let ecl = ec_level_of(params.ec_level);
    let version = qrcode::Version::Normal(params.version as i16);
    let bits = Bits::new(version);
    let max_length = bits.max_len(ecl)? / 8;
    let max_length = max_length - 20; // header size = raptor params 12 + qr header 4 + packet header 4
    let encoder = Encoder::with_defaults(&data, max_length as u16);
    let header = encoder.get_config().serialize();
    let packets = encoder.get_encoded_packets(params.repair);
    let ser_packets = packets
        .iter()
        .map(|p| {
            let mut packet = header.to_vec();
            packet.extend(p.serialize());
            packet
        })
        .collect::<Vec<_>>();

    Ok(ser_packets)
}

#[frb(sync)]
pub fn get_qr_bytes(data: &[u8]) -> Result<Vec<u8>> {
    let mut v = vec![];
    v.reserve(data.len());
    let len = data.len();
    for i in 0..len {
        let c = if i + 1 < len { data[i + 1] >> 4 } else { 0 };
        v.push((data[i] << 4) | c);
    }
    let len = (u16::from_be_bytes(v[0..2].try_into().unwrap())) as usize;
    v.rotate_left(2);
    v.truncate(len);
    Ok(v)
}

#[frb]
pub async fn decode(packet: &[u8]) -> Result<Option<Vec<u8>>> {
    {
        let mut dec = DECODER.lock().unwrap();
        if dec.is_none() {
            let oti = ObjectTransmissionInformation::deserialize(packet[0..12].try_into().unwrap());
            let decoder = Decoder::new(oti);
            *dec = Some(decoder);
        }
    }

    let mut dec = DECODER.lock().unwrap();
    let decoder = dec.as_mut().unwrap();
    let packet = EncodingPacket::deserialize(&packet[12..]);
    let result = decoder.decode(packet);

    Ok(result)
}

pub async fn end_decode() -> Result<()> {
    let mut decoder = DECODER.lock().unwrap();
    *decoder = None;
    Ok(())
}

fn ec_level_of(level: u8) -> EcLevel {
    match level {
        0 => EcLevel::L,
        1 => EcLevel::M,
        2 => EcLevel::Q,
        3 => EcLevel::H,
        _ => unreachable!(),
    }
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    // Default utilities - feel free to customize
    flutter_rust_bridge::setup_default_user_utils();
}

pub static DECODER: LazyLock<Mutex<Option<Decoder>>> =
    LazyLock::new(|| Mutex::new(None));

