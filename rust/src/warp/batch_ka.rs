//! Orchard and Ironwood key agreement for many actions at once, with
//! `zakura-udon`.
//!
//! Trial decryption multiplies every action's ephemeral key by the same
//! incoming viewing key. Here a chunk's keys are decoded together, the scalar
//! is recoded once, the multiplications run as synchronized ladders sharing
//! their field inversions, and the products are normalized together.
//!
//! The arithmetic is variable-time in the scalar, here the wallet's incoming
//! viewing key: its timing and memory access depend on it.

use zakura_udon::{
    curve::{
        batch_mul_same_scalar_prepared, batch_normalize, same_scalar_scratch, EisensteinScalar,
        Pallas, PallasPoint, PallasProjective,
    },
    exec::{SerialExecutor, TaskBudget},
    field::{Fp, Fq},
};

/// The key agreement of the scalar `ivk` with each ephemeral key, encoded as
/// the KDF hashes it (as [`super::orchard_shared_secret`] does);
/// None for an encoding that is not a point. None overall when `ivk` is not
/// a canonical scalar.
pub fn orchard_shared_secrets(ivk: &[u8; 32], epks: &[[u8; 32]]) -> Option<Vec<Option<[u8; 32]>>> {
    let scalar = EisensteinScalar::<Pallas>::new(&Fq::from_bytes(*ivk)?);
    let mut decoded = vec![PallasPoint::IDENTITY; epks.len()];
    let mut valid = vec![true; epks.len()];
    if PallasPoint::batch_from_bytes(epks, &mut decoded).is_none() {
        // One encoding was rejected: decode them one by one to find it.
        for ((epk, point), valid) in epks.iter().zip(decoded.iter_mut()).zip(valid.iter_mut()) {
            match PallasPoint::from_bytes(*epk) {
                Some(p) => *point = p,
                None => {
                    *point = PallasPoint::IDENTITY;
                    *valid = false;
                }
            }
        }
    }
    let mut points: Vec<PallasProjective> =
        decoded.iter().map(PallasProjective::from_point).collect();
    let mut scratch = vec![Fp::ZERO; same_scalar_scratch::<Pallas>(points.len()).ok()?];
    batch_mul_same_scalar_prepared(&mut points, &scalar, &mut scratch, TaskBudget::SERIAL, &SerialExecutor);
    let mut products = decoded;
    let mut scratch = vec![Fp::ZERO; points.len()];
    batch_normalize(&points, &mut products, &mut scratch);
    Some(products.iter().zip(valid).map(|(p, valid)| valid.then(|| p.to_bytes())).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use halo2_proofs::pasta::{
        group::{ff::PrimeField as _, Curve, Group as _, GroupEncoding as _},
        pallas::Point,
        Fq as PastaFq,
    };

    /// The batched secrets are byte for byte the ones computed one at a time,
    /// identity and undecodable keys included.
    #[test]
    fn batched_secrets_match_one_at_a_time() {
        let ivk = PastaFq::from(0x1234_5678_9abc_def0u64) * PastaFq::from(0x0fed_cba9_8765u64);
        let mut epks: Vec<[u8; 32]> = (1..300u64)
            .map(|i| (Point::generator() * PastaFq::from(i * 7919 + 3)).to_affine().to_bytes())
            .collect();
        epks.push([0u8; 32]); // identity
        epks.push([0xffu8; 32]); // not a point
        let batched = orchard_shared_secrets(&ivk.to_repr(), &epks).unwrap();
        for (epk, secret) in epks.iter().zip(&batched) {
            let expected = Option::<Point>::from(Point::from_bytes(epk))
                .map(|epk| (epk.to_affine() * ivk).to_bytes());
            assert_eq!(*secret, expected);
        }
    }
}
