use std::{
    fs::File,
    io::Read,
    sync::{LazyLock, Mutex},
};

use anyhow::Result;
#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;
use qrcode::{bits::Bits, EcLevel};
use raptorq::{Decoder, Encoder, EncodingPacket, ObjectTransmissionInformation};

pub struct RaptorQParams {
    pub version: u16,
    pub ec_level: u8,
    pub repair: u32,
}

#[cfg_attr(feature = "flutter", frb)]
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

#[cfg_attr(feature = "flutter", frb(sync))]
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

/// A QR frame: the 12-byte transmission header, then the packet (a 4-byte
/// payload id and the symbol), so at least 16 bytes.
const FRAME_MIN_BYTES: usize = 16;

/// Feed one scanned frame to the decoder of the transfer in progress and
/// return the object once it is complete (#1504).
///
/// The first frame of a scan configures the decoder from its header; a
/// later frame with another header belongs to another transfer and is an
/// error, not a reset: re-creating the decoder would silently discard the
/// frames collected so far. A completed transfer clears the decoder, so
/// the next scan starts fresh even if the page is not disposed. A frame
/// too short to carry a packet, or a header claiming more than the
/// largest PCZT the wallet accepts, is refused before any decoder exists.
#[cfg_attr(feature = "flutter", frb)]
pub async fn decode(packet: &[u8]) -> Result<Option<Vec<u8>>> {
    if packet.len() < FRAME_MIN_BYTES {
        anyhow::bail!(
            "QR frame is {} bytes, at least {FRAME_MIN_BYTES} expected",
            packet.len()
        );
    }
    let header: [u8; 12] = packet[0..12].try_into().unwrap();
    let oti = ObjectTransmissionInformation::deserialize(&header);
    if oti.transfer_length() > crate::api::pay::PCZT_MAX_BYTES as u64 {
        anyhow::bail!(
            "QR transfer claims {} bytes, more than the {} the wallet accepts",
            oti.transfer_length(),
            crate::api::pay::PCZT_MAX_BYTES
        );
    }

    let mut dec = DECODER.lock().unwrap();
    let (stored, decoder) = dec.get_or_insert_with(|| (header, Decoder::new(oti)));
    if *stored != header {
        anyhow::bail!("QR frame belongs to a different transfer");
    }
    let packet = EncodingPacket::deserialize(&packet[12..]);
    let result = decoder.decode(packet);
    if result.is_some() {
        *dec = None;
    }
    Ok(result)
}

/// Forget the transfer in progress: the scanner page calls it when a scan
/// starts and when it is left, so no scan inherits another's frames.
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

#[cfg(feature = "flutter")]
#[frb(init)]
pub fn init_app() {
    // Default utilities - feel free to customize
    flutter_rust_bridge::setup_default_user_utils();
}

/// The transfer in progress: the header its decoder was built from, and
/// the decoder.
type Transfer = ([u8; 12], Decoder);

pub static DECODER: LazyLock<Mutex<Option<Transfer>>> = LazyLock::new(|| Mutex::new(None));

#[cfg(test)]
mod tests {
    use super::*;

    fn frames(payload: &[u8]) -> Vec<Vec<u8>> {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("raptor-{}-{}.bin", std::process::id(), payload[0]));
        std::fs::write(&path, payload).unwrap();
        let params = RaptorQParams {
            version: 10,
            ec_level: 0,
            repair: 4,
        };
        let frames = futures::executor::block_on(encode(path.to_str().unwrap(), params)).unwrap();
        let _ = std::fs::remove_file(path);
        frames
    }

    fn feed(frame: &[u8]) -> Result<Option<Vec<u8>>> {
        futures::executor::block_on(decode(frame))
    }

    /// One scenario, in order, because the decoder is process-wide.
    #[test]
    fn a_foreign_frame_is_an_error_and_a_completed_transfer_clears_the_decoder() {
        futures::executor::block_on(end_decode()).unwrap();
        let a: Vec<u8> = (0..500u32).map(|i| (i * 7) as u8).collect();
        let b: Vec<u8> = (0..700u32).map(|i| (i * 3 + 1) as u8).collect();
        let frames_a = frames(&a);
        let frames_b = frames(&b);

        assert_eq!(feed(&frames_a[0]).unwrap(), None);
        let err = feed(&frames_b[0]).unwrap_err();
        assert!(err.to_string().contains("different transfer"), "{err}");

        let mut done = None;
        for frame in &frames_a[1..] {
            if let Some(object) = feed(frame).unwrap() {
                done = Some(object);
                break;
            }
        }
        assert_eq!(done.as_deref(), Some(a.as_slice()), "the first transfer still completes");

        let mut done = None;
        for frame in &frames_b {
            if let Some(object) = feed(frame).unwrap() {
                done = Some(object);
                break;
            }
        }
        assert_eq!(done.as_deref(), Some(b.as_slice()), "a new transfer starts fresh after a completed one");
        futures::executor::block_on(end_decode()).unwrap();
    }

    #[test]
    fn a_short_frame_is_refused() {
        let err = feed(&[0u8; 10]).unwrap_err();
        assert!(err.to_string().contains("10 bytes"), "{err}");
    }

    #[test]
    fn a_header_claiming_more_than_a_pczt_may_weigh_is_refused() {
        let oti = ObjectTransmissionInformation::new(
            (crate::api::pay::PCZT_MAX_BYTES as u64) + 1,
            1000,
            1,
            1,
            8,
        );
        let mut frame = oti.serialize().to_vec();
        frame.extend_from_slice(&[0u8; 8]);
        let err = feed(&frame).unwrap_err();
        assert!(err.to_string().contains("more than"), "{err}");
    }
}
