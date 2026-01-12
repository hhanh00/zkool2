use std::collections::BTreeMap;

use anyhow::Result;
use bincode::{config, Decode, Encode};
use reddsa::frost::redpallas::{
    keys::dkg::{round1, round2},
    Identifier, PallasBlake2b512,
};

pub type P = PallasBlake2b512;

pub mod db;
pub mod dkg;
pub mod sign;

pub type PK1Map = BTreeMap<Identifier, round1::Package>;
pub type PK2Map = BTreeMap<Identifier, round2::Package>;

#[derive(Encode, Decode)]
pub struct FrostMessage {
    pub from_id: u8,
    pub data: Vec<u8>,
}

impl FrostMessage {
    pub fn encode_with_prefix(&self, prefix: &[u8]) -> Result<Vec<u8>> {
        let mut data = vec![];
        data.extend_from_slice(prefix);
        bincode::encode_into_std_write(self, &mut data, config::legacy())?;
        Ok(data)
    }
}

#[derive(Encode, Decode)]
pub struct FrostSigMessage {
    pub sighash: [u8; 32],
    pub from_id: u16,
    pub idx: u32,
    pub data: Vec<u8>,
}

impl FrostSigMessage {
    pub fn encode_with_prefix(&self, prefix: &[u8]) -> Result<Vec<u8>> {
        let mut data = vec![];
        data.extend_from_slice(prefix);
        bincode::encode_into_std_write(self, &mut data, config::legacy())?;
        Ok(data)
    }
}

pub fn to_arb_memo(pk1: &[u8]) -> Vec<u8> {
    let mut memo_bytes = vec![0xFF];
    memo_bytes.extend_from_slice(pk1);
    memo_bytes
}
