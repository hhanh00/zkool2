mod decrypter;
pub mod edge;
pub mod hasher;
pub mod legacy;
mod orchard;
mod sapling;
pub mod sync;
pub mod witnesses;

use crate::{lwd::CompactBlock, Hash32};
use bincode::{Decode, Encode};
use secp256k1::SecretKey;

pub(crate) const MERKLE_DEPTH: u8 = 32;

#[derive(Clone, Default, Encode, Decode, PartialEq, Debug)]
pub struct Edge(pub [Option<Hash32>; MERKLE_DEPTH as usize]);

#[derive(Encode, Decode, Default, Debug)]
pub struct AuthPath(pub [Hash32; MERKLE_DEPTH as usize]);

#[derive(Clone, Default, Encode, Decode, PartialEq, Debug)]
pub struct Witness {
    pub value: Hash32,
    pub position: u32,
    pub ommers: Edge,
    pub anchor: Hash32, // for debugging
}

impl std::fmt::Display for Witness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.position, hex::encode(self.value), hex::encode(self.anchor))
    }
}

#[derive(Clone, Default, Encode, Decode, Debug)]
pub struct BlockHeader {
    pub height: u32,
    pub hash: Hash32,
    pub prev_hash: Hash32,
    pub timestamp: u32,
}

impl From<&CompactBlock> for BlockHeader {
    fn from(block: &CompactBlock) -> Self {
        BlockHeader {
            height: block.height as u32,
            hash: block.hash.clone().try_into().unwrap(),
            prev_hash: block.prev_hash.clone().try_into().unwrap(),
            timestamp: block.time,
        }
    }
}

pub trait Hasher: std::fmt::Debug + Default {
    fn empty(&self) -> Hash32;
    fn combine(&self, depth: u8, l: &Hash32, r: &Hash32) -> Hash32;
    fn parallel_combine(&self, depth: u8, layer: &[Hash32], pairs: usize) -> Vec<Hash32>;
    fn parallel_combine_opt(
        &self,
        depth: u8,
        layer: &[Option<Hash32>],
        pairs: usize,
    ) -> Vec<Option<Hash32>>;
}

#[derive(Clone, Default, Encode, Decode, Debug)]
pub struct OutPoint {
    pub txid: Hash32,
    pub vout: u32,
}

#[derive(Default, Debug)]
pub struct TxOut {
    pub address: Option<TransparentAddress>,
    pub value: u64,
    pub vout: u32,
}

#[derive(Clone, Default, Encode, Decode, Debug)]
pub struct TxOut2 {
    pub address: Option<String>,
    pub value: u64,
    pub vout: u32,
}

#[derive(Debug)]
pub struct TransparentTx {
    pub account: u32,
    pub external: u32,
    pub addr_index: u32,
    pub address: TransparentAddress,
    pub height: u32,
    pub timestamp: u32,
    pub txid: Hash32,
    pub vins: Vec<OutPoint>,
    pub vouts: Vec<TxOut>,
}

#[derive(Debug)]
pub struct STXO {
    pub account: u32,
    pub txid: Hash32,
    pub vout: u32,
    pub address: String,
    pub value: u64,
}

#[derive(Debug)]
pub struct UTXO {
    pub is_new: bool,
    pub id: u32,
    pub account: u32,
    pub external: u32,
    pub addr_index: u32,
    pub height: u32,
    pub timestamp: u32,
    pub txid: Hash32,
    pub vout: u32,
    pub address: String,
    pub value: u64,
}

#[derive(Debug)]
pub struct TransparentSK {
    pub address: String,
    pub sk: SecretKey,
}

pub use decrypter::{try_orchard_decrypt, try_sapling_decrypt};
use zcash_primitives::legacy::TransparentAddress;
