use halo2_proofs::pasta::pallas::Point;
use jubjub::Fr;

use super::{AuthPath, Hash32, Hasher, MERKLE_DEPTH};

#[derive(Default, Debug)]
pub struct SaplingHasher {}

impl Hasher for SaplingHasher {
    fn empty(&self) -> Hash32 {
        Fr::one().to_bytes()
    }

    fn combine(&self, depth: u8, l: &Hash32, r: &Hash32) -> Hash32 {
        super::sapling::hash_combine(depth, l, r)
    }

    fn parallel_combine(&self, depth: u8, layer: &[Hash32], pairs: usize) -> Vec<Hash32> {
        super::sapling::parallel_hash(depth, layer, pairs)
    }

    fn parallel_combine_opt(
        &self,
        depth: u8,
        layer: &[Option<Hash32>],
        pairs: usize,
    ) -> Vec<Option<Hash32>> {
        super::sapling::parallel_hash_opt(depth, layer, pairs)
    }
}

#[derive(Debug)]
pub struct OrchardHasher {
    pub(crate) q: Point,
}

pub fn empty_roots<H: Hasher>(h: &H) -> AuthPath {
    let mut empty = h.empty();
    let mut empty_roots = AuthPath::default();
    for i in 0..MERKLE_DEPTH as usize {
        empty_roots.0[i] = empty;
        empty = h.combine(i as u8, &empty, &empty);
    }
    empty_roots
}
