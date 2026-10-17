//! The public key package a coordinator aggregates with, rebuilt from the
//! verifying shares the signing participants send with their commitments.
//!
//! A member restored from its backup string has a key package but no
//! public key package: the DKG's round-3 row was never on its device. The
//! shares of the t participants, together with the group `ak` from the
//! account's viewing key, are enough to rebuild one, provided the shares
//! interpolate to `ak`; that check is what stops a wrong share from being
//! trusted.

use std::collections::BTreeMap;

use anyhow::{Context, Result};
use ff_legacy::{Field, PrimeField};
use pasta_curves_legacy::{
    group::{Group, GroupEncoding},
    pallas,
};
use reddsa_legacy::frost::redpallas::{
    frost::keys::{PublicKeyPackage, VerifyingShare},
    frost::VerifyingKey,
    Identifier,
};

use crate::frost::P;

/// Lagrange-interpolate the verifying shares at zero.
fn interpolate(shares: &BTreeMap<Identifier, VerifyingShare<P>>) -> Result<pallas::Point> {
    let mut points = Vec::with_capacity(shares.len());
    for (id, share) in shares {
        let x: [u8; 32] = id
            .serialize()
            .try_into()
            .map_err(|_| anyhow::anyhow!("identifier is not 32 bytes"))?;
        let x: pallas::Scalar =
            Option::from(pallas::Scalar::from_repr(x)).context("identifier scalar")?;
        let p: [u8; 32] = share
            .serialize()?
            .try_into()
            .map_err(|_| anyhow::anyhow!("verifying share is not 32 bytes"))?;
        let p: pallas::Point =
            Option::from(pallas::Point::from_bytes(&p)).context("verifying share point")?;
        points.push((x, p));
    }
    let mut acc = pallas::Point::identity();
    for (i, (x_i, p_i)) in points.iter().enumerate() {
        let mut num = pallas::Scalar::ONE;
        let mut den = pallas::Scalar::ONE;
        for (j, (x_j, _)) in points.iter().enumerate() {
            if i != j {
                num *= x_j;
                den *= x_j - x_i;
            }
        }
        let inv: pallas::Scalar = Option::from(den.invert()).context("duplicate identifier")?;
        acc += *p_i * (num * inv);
    }
    Ok(acc)
}

/// Build the public key package from the signing set's verifying shares and
/// the group key `ak` (its 32-byte even-y encoding), after checking that the
/// shares interpolate to `ak`.
pub fn rebuild_public_key_package(
    shares: BTreeMap<Identifier, VerifyingShare<P>>,
    ak: &[u8; 32],
) -> Result<PublicKeyPackage<P>> {
    anyhow::ensure!(!shares.is_empty(), "no verifying shares");
    let interpolated = interpolate(&shares)?;
    anyhow::ensure!(
        interpolated.to_bytes() == *ak,
        "the verifying shares do not interpolate to the group key; a share is wrong"
    );
    let verifying_key = VerifyingKey::deserialize(ak)?;
    Ok(PublicKeyPackage::new(shares, verifying_key))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_core_legacy::OsRng;
    use reddsa_legacy::frost::redpallas::keys::{generate_with_dealer, IdentifierList, KeyPackage};

    /// Any t of the n shares rebuild the dealer's package; a wrong share is
    /// caught by the interpolation check.
    #[test]
    fn t_shares_rebuild_the_package_and_a_wrong_share_is_rejected() {
        let (secret_shares, pubkeys) =
            generate_with_dealer(3, 2, IdentifierList::Default, OsRng).unwrap();
        let ak: [u8; 32] = pubkeys.verifying_key().serialize().unwrap().try_into().unwrap();
        let key_packages: BTreeMap<_, _> = secret_shares
            .iter()
            .map(|(id, s)| (*id, KeyPackage::try_from(s.clone()).unwrap()))
            .collect();
        let ids: Vec<Identifier> = key_packages.keys().copied().collect();

        for pair in [[ids[0], ids[1]], [ids[0], ids[2]], [ids[1], ids[2]]] {
            let shares: BTreeMap<_, _> = pair
                .iter()
                .map(|id| (*id, *key_packages[id].verifying_share()))
                .collect();
            let rebuilt = rebuild_public_key_package(shares.clone(), &ak).unwrap();
            assert_eq!(rebuilt.verifying_key(), pubkeys.verifying_key());
            for id in pair {
                assert_eq!(
                    rebuilt.verifying_shares()[&id],
                    pubkeys.verifying_shares()[&id]
                );
            }
        }

        // One share swapped for another participant's: wrong, rejected.
        let mut wrong = BTreeMap::new();
        wrong.insert(ids[0], *key_packages[&ids[0]].verifying_share());
        wrong.insert(ids[1], *key_packages[&ids[2]].verifying_share());
        assert!(rebuild_public_key_package(wrong, &ak).is_err());
    }
}
