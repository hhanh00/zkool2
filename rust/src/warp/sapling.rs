use crate::Hash32;
use halo2_proofs::pasta::group::{ff::PrimeField as _, Curve as _, GroupEncoding as _};
use jubjub::{AffinePoint, ExtendedNielsPoint, ExtendedPoint, Fr, SubgroupPoint};
use rayon::prelude::*;
use sapling_crypto::constants::PEDERSEN_HASH_CHUNKS_PER_GENERATOR;
use std::{io::Read, sync::LazyLock};

static GENERATORS_EXP: LazyLock<Vec<ExtendedNielsPoint>> = LazyLock::new(read_generators_bin);

pub const GENERATORS: &[u8] = include_bytes!("generators.bin");

#[inline(always)]
fn accumulate_scalar(acc: &mut Fr, cur: &mut Fr, x: u8) {
    let mut tmp = *cur;
    if x & 1 != 0 {
        tmp += *cur;
    }
    *cur = cur.double();
    if x & 2 != 0 {
        tmp += *cur;
    }
    if x & 4 != 0 {
        tmp = tmp.neg();
    }

    *acc += tmp;
}

fn accumulate_generator(acc: &Fr, idx_generator: u32) -> ExtendedPoint {
    let acc_bytes = acc.to_repr();

    let mut tmp = ExtendedPoint::identity();
    for (i, &j) in acc_bytes.iter().enumerate() {
        let offset = (idx_generator * 32 + i as u32) * 256 + j as u32;
        let x = GENERATORS_EXP[offset as usize];
        tmp += x;
    }
    tmp
}

pub fn hash_combine(depth: u8, left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    // info!("+ {} {} {}", depth, hex::encode(left), hex::encode(right));
    let p = hash_combine_inner(depth, left, right);
    p.to_affine().get_u().to_repr()
}

pub fn hash_combine_inner(depth: u8, left: &[u8; 32], right: &[u8; 32]) -> ExtendedPoint {
    let mut hash = ExtendedPoint::identity();
    let mut acc = Fr::zero();
    let mut cur = Fr::one();

    let a = depth & 7;
    let b = depth >> 3;

    accumulate_scalar(&mut acc, &mut cur, a);
    cur = cur.double().double().double();
    accumulate_scalar(&mut acc, &mut cur, b);
    cur = cur.double().double().double();

    // Shift right by 1 bit and overwrite the 256th bit of left
    let mut left = *left;
    let mut right = *right;

    // move by 1 bit to fill the missing 256th bit of left
    let mut carry = 0;
    for i in (0..32).rev() {
        let c = right[i] & 1;
        right[i] = right[i] >> 1 | carry << 7;
        carry = c;
    }
    left[31] &= 0x7F;
    left[31] |= carry << 7; // move the first bit of right into 256th of left

    // we have 255*2/3 = 170 chunks
    let mut bit_offset = 0;
    let mut byte_offset = 0;
    let mut idx_generator = 0;
    for i in 0..170 {
        let mut v = if byte_offset < 31 {
            left[byte_offset] as u16 | (left[byte_offset + 1] as u16) << 8
        } else if byte_offset == 31 {
            left[31] as u16 | (right[0] as u16) << 8
        } else if byte_offset < 63 {
            right[byte_offset - 32] as u16 | (right[byte_offset - 31] as u16) << 8
        } else {
            right[byte_offset - 32] as u16
        };
        v = v >> bit_offset & 0x07; // keep 3 bits
        accumulate_scalar(&mut acc, &mut cur, v as u8);

        if (i + 3) % PEDERSEN_HASH_CHUNKS_PER_GENERATOR as u32 == 0 {
            hash += accumulate_generator(&acc, idx_generator);
            idx_generator += 1;
            acc = Fr::zero();
            cur = Fr::one();
        } else {
            cur = cur.double().double().double(); // 2^4 * cur
        }
        bit_offset += 3;
        if bit_offset >= 8 {
            byte_offset += bit_offset / 8;
            bit_offset %= 8;
        }
    }
    hash += accumulate_generator(&acc, idx_generator);

    hash
}

pub fn parallel_hash(depth: u8, layer: &[[u8; 32]], pairs: usize) -> Vec<[u8; 32]> {
    let hash_extended: Vec<_> = (0..pairs)
        .into_par_iter()
        .map(|i| hash_combine_inner(depth, &layer[2 * i], &layer[2 * i + 1]))
        .collect();
    hash_normalize(&hash_extended)
}

fn hash_normalize(extended: &[ExtendedPoint]) -> Vec<[u8; 32]> {
    let mut hash_affine = vec![AffinePoint::identity(); extended.len()];
    ExtendedPoint::batch_normalize(extended, &mut hash_affine);
    hash_affine.iter().map(|p| p.get_u().to_repr()).collect()
}

pub fn parallel_hash_opt(depth: u8, layer: &[Option<Hash32>], pairs: usize) -> Vec<Option<Hash32>> {
    let hash_extended: Vec<Option<ExtendedPoint>> = (0..pairs)
        .into_par_iter()
        .map(|i| {
            let l = &layer[2 * i];
            let r = &layer[2 * i + 1];
            match (l, r) {
                (Some(l), Some(r)) => Some(hash_combine_inner(depth, l, r)),
                _ => None,
            }
        })
        .collect();

    let ext = hash_extended.iter().flatten().cloned().collect::<Vec<_>>();
    let mut hash_affine = vec![AffinePoint::identity(); ext.len()];
    ExtendedPoint::batch_normalize(&ext, &mut hash_affine);
    let mut h_cursor = hash_affine.iter();

    hash_extended
        .iter()
        .map(|n| {
            n.map(|_| {
                let ep = h_cursor.next().unwrap();
                ep.get_u().to_repr()
            })
        })
        .collect::<Vec<_>>()
}

fn read_generators_bin() -> Vec<ExtendedNielsPoint> {
    let mut generators_bin = GENERATORS;
    let mut gens: Vec<ExtendedNielsPoint> = vec![];
    gens.reserve_exact(3 * 32 * 256);
    for _i in 0..3 {
        for _j in 0..32 {
            for _k in 0..256 {
                let mut bb = [0u8; 32];
                generators_bin.read_exact(&mut bb).unwrap();
                let p = ExtendedPoint::from(SubgroupPoint::from_bytes_unchecked(&bb).unwrap())
                    .to_niels();
                gens.push(p);
            }
        }
    }
    gens
}
