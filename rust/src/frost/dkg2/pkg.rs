use std::collections::BTreeMap;

use anyhow::Result;
use rand_core::OsRng;
use reddsa::frost::redpallas::keys::{KeyPackage, PublicKeyPackage};
use reddsa::frost::redpallas::keys::dkg::round1::{Package as P1P, SecretPackage as S1P};
use reddsa::frost::redpallas::keys::dkg::round2::{Package as P2P, SecretPackage as S2P};
use reddsa::frost::redpallas::Identifier;
use either::Either;

use super::protocol::{BinaryRW, SecretData, PublicData, FrostMap};
use crate::tiu;

#[derive(Clone, Debug)]
pub struct DKGRound1Secret(pub S1P);

#[derive(Clone, Debug)]
pub struct DKGRound1Public(pub P1P);

pub fn build_part1(id: u8, n: u8, t: u8) -> Result<(DKGRound1Secret, Either<DKGRound1Public, FrostMap<DKGRound1Public>>)> {
    let (s, p) = reddsa::frost::redpallas::keys::dkg::part1::<_>(
        tiu!(id as u16),
        n as u16,
        t as u16,
        OsRng,
    )?;
    Ok((DKGRound1Secret(s), Either::Left(DKGRound1Public(p))))
}

#[derive(Clone, Debug)]
pub struct DKGRound2Secret(pub S2P);

#[derive(Clone, Debug)]
pub struct DKGRound2Public(pub P2P);

pub fn build_part2(
    secret_package: DKGRound1Secret,
    round1_packages: BTreeMap<Identifier, DKGRound1Public>,
) -> Result<(DKGRound2Secret, Either<DKGRound2Public, FrostMap<DKGRound2Public>>)> {
    let DKGRound1Secret(secret_package) = secret_package;
    let round1_packages: BTreeMap<_, _> =
        round1_packages.into_iter().map(|(i, p)| (i, p.0)).collect();
    let (s, p) = reddsa::frost::redpallas::keys::dkg::part2(secret_package, &round1_packages)?;
    let p = p.into_iter().map(|(i, p)| (i, DKGRound2Public(p))).collect();
    Ok((DKGRound2Secret(s), Either::Right(p)))
}

#[derive(Clone, Debug)]
pub struct DKGRound3Secret(pub KeyPackage);

#[derive(Clone, Debug)]
pub struct DKGRound3Public(pub PublicKeyPackage);

pub fn build_part3(
    secret_package: DKGRound2Secret,
    round1_packages: BTreeMap<Identifier, DKGRound1Public>,
    round2_packages: BTreeMap<Identifier, DKGRound2Public>,
) -> Result<(DKGRound3Secret, Either<DKGRound3Public, FrostMap<DKGRound3Public>>)> {
    let DKGRound2Secret(secret_package) = secret_package;
    let round1_packages: BTreeMap<_, _> =
        round1_packages.into_iter().map(|(i, p)| (i, p.0)).collect();
    let round2_packages: BTreeMap<_, _> =
        round2_packages.into_iter().map(|(i, p)| (i, p.0)).collect();
    let (s, p) = reddsa::frost::redpallas::keys::dkg::part3(&secret_package, &round1_packages, &round2_packages)?;
    Ok((DKGRound3Secret(s), Either::Left(DKGRound3Public(p))))
}

impl SecretData for DKGRound1Secret {
    type Public = DKGRound1Public;
}

impl BinaryRW for DKGRound1Secret {
    fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        let s = S1P::deserialize(bytes)?;
        Ok(Self(s))
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.0.serialize().expect("Must be serializable")
    }
}

impl PublicData for DKGRound1Public {
    fn can_broadcast() -> bool {
        true
    }
}

impl BinaryRW for DKGRound1Public {
    fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        let p = P1P::deserialize(bytes)?;
        Ok(DKGRound1Public(p))
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.0
            .serialize()
            .expect("DKG Public Round 1 must be serializable")
    }
}

impl SecretData for DKGRound2Secret {
    type Public = DKGRound2Public;
}

impl PublicData for DKGRound2Public {
    fn can_broadcast() -> bool {
        false
    }
}

impl BinaryRW for DKGRound2Secret {
    fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        let p = S2P::deserialize(bytes)?;
        Ok(DKGRound2Secret(p))
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.0.serialize().expect("Must be serializable")
    }
}

impl BinaryRW for DKGRound2Public {
    fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        let p = P2P::deserialize(bytes)?;
        Ok(DKGRound2Public(p))
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.0.serialize().expect("Must be serializable")
    }
}

impl SecretData for DKGRound3Secret {
    type Public = DKGRound3Public;
}

impl PublicData for DKGRound3Public {
    fn can_broadcast() -> bool {
        unreachable!()
    }
}

impl BinaryRW for DKGRound3Secret {
    fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        let p = KeyPackage::deserialize(bytes)?;
        Ok(DKGRound3Secret(p))
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.0.serialize().expect("Must be serializable")
    }
}

impl BinaryRW for DKGRound3Public {
    fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        let p = PublicKeyPackage::deserialize(bytes)?;
        Ok(DKGRound3Public(p))
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.0.serialize().expect("Must be serializable")
    }
}

