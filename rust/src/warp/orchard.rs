use super::{hasher::OrchardHasher, Hash32, Hasher};
use halo2_gadgets::sinsemilla::primitives::SINSEMILLA_S;
use halo2_proofs::pasta::group::{ff::PrimeField as _, prime::PrimeCurveAffine as _, Curve as _};
use halo2_proofs::{
    arithmetic::{CurveAffine as _, CurveExt as _},
    pasta::{
        pallas::{self, Affine, Point},
        Ep, EpAffine, Fp, Fq,
    },
};
use rayon::prelude::*;

impl OrchardHasher {
    fn node_combine_inner(&self, depth: u8, left: &Hash32, right: &Hash32) -> Point {
        let mut acc = self.q;
        let (s_x, s_y) = SINSEMILLA_S[depth as usize];
        let s_chunk = Affine::from_xy(s_x, s_y).unwrap();
        acc = (acc + s_chunk) + acc; // TODO Bail if + gives point at infinity? Shouldn't happen if data was validated

        // Shift right by 1 bit and overwrite the 256th bit of left
        let mut left = *left;
        let mut right = *right;
        left[31] |= (right[0] & 1) << 7; // move the first bit of right into 256th of left
        for i in 0..32 {
            // move by 1 bit to fill the missing 256th bit of left
            let carry = if i < 31 { (right[i + 1] & 1) << 7 } else { 0 };
            right[i] = right[i] >> 1 | carry;
        }

        // we have 255*2/10 = 51 chunks
        let mut bit_offset = 0;
        let mut byte_offset = 0;
        for _ in 0..51 {
            let mut v = if byte_offset < 31 {
                left[byte_offset] as u16 | (left[byte_offset + 1] as u16) << 8
            } else if byte_offset == 31 {
                left[31] as u16 | (right[0] as u16) << 8
            } else {
                right[byte_offset - 32] as u16 | (right[byte_offset - 31] as u16) << 8
            };
            v = v >> bit_offset & 0x03FF; // keep 10 bits
            let (s_x, s_y) = SINSEMILLA_S[v as usize];
            let s_chunk = Affine::from_xy(s_x, s_y).unwrap();
            acc = (acc + s_chunk) + acc;
            bit_offset += 10;
            if bit_offset >= 8 {
                byte_offset += bit_offset / 8;
                bit_offset %= 8;
            }
        }
        acc
    }
}

impl Default for OrchardHasher {
    fn default() -> Self {
        let q = Point::hash_to_curve(halo2_gadgets::sinsemilla::primitives::Q_PERSONALIZATION)(
            halo2_gadgets::sinsemilla::merkle::MERKLE_CRH_PERSONALIZATION.as_bytes(),
        );
        Self { q }
    }
}

impl Hasher for OrchardHasher {
    fn empty(&self) -> crate::Hash32 {
        Fq::from(2).to_repr()
    }

    fn combine(&self, depth: u8, l: &crate::Hash32, r: &crate::Hash32) -> crate::Hash32 {
        let acc = self.node_combine_inner(depth, l, r);
        let p = acc
            .to_affine()
            .coordinates()
            .map(|c| *c.x())
            .unwrap_or_else(Fp::zero);
        p.to_repr()
    }

    fn parallel_combine(
        &self,
        depth: u8,
        layer: &[crate::Hash32],
        pairs: usize,
    ) -> Vec<crate::Hash32> {
        let hash_extended: Vec<_> = (0..pairs)
            .into_par_iter()
            .map(|i| self.node_combine_inner(depth, &layer[2 * i], &layer[2 * i + 1]))
            .collect();
        let mut hash_affine = vec![EpAffine::identity(); hash_extended.len()];
        Point::batch_normalize(&hash_extended, &mut hash_affine);
        hash_affine
            .iter()
            .map(|p| {
                p.coordinates()
                    .map(|c| *c.x())
                    .unwrap_or_else(pallas::Base::zero)
                    .to_repr()
            })
            .collect()
    }

    fn parallel_combine_opt(
        &self,
        depth: u8,
        layer: &[Option<Hash32>],
        pairs: usize,
    ) -> Vec<Option<Hash32>> {
        let hash_extended: Vec<Option<Ep>> = (0..pairs)
            .into_par_iter()
            .map(|i| match (&layer[2 * i], &layer[2 * i + 1]) {
                (Some(l), Some(r)) => Some(self.node_combine_inner(depth, l, r)),
                _ => None,
            })
            .collect();
        let ext = hash_extended.iter().flatten().cloned().collect::<Vec<_>>();
        let mut hash_affine = vec![EpAffine::identity(); ext.len()];
        Point::batch_normalize(&ext, &mut hash_affine);
        let mut h_cursor = hash_affine.iter();
        hash_extended
            .iter()
            .map(|n| {
                n.map(|_| {
                    h_cursor
                        .next()
                        .unwrap()
                        .coordinates()
                        .map(|c| *c.x())
                        .unwrap_or_else(pallas::Base::zero)
                        .to_repr()
                })
            })
            .collect::<Vec<_>>()
    }
}
