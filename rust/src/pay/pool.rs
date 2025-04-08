use super::error::{Error, Result};
use zcash_address::ZcashAddress;
use zcash_protocol::PoolType;

// A bit field to represent the pools that a transaction can use
// in a single byte. The bits are as follows:
// 0 - Transparent
// 1 - Sapling
// 2 - Orchard
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub struct PoolMask(pub u8);

pub const ALL_POOLS: u8 = 0b111;

impl PoolMask {
    pub fn empty() -> Self {
        PoolMask(0)
    }

    // Create a new PoolMask from a single bit value
    pub fn from_pool(pool: u8) -> Self {
        Self(1 << pool)
    }

    // Return the best pool available in the mask
    // or None if no pools are available
    pub fn to_best_pool(&self) -> Option<u8> {
        if self.0 & 4 != 0 {
            return Some(2);
        }
        if self.0 & 2 != 0 {
            return Some(1);
        }
        if self.0 & 1 != 0 {
            return Some(0);
        }
        None
    }

    // Return true if the pool mask has a single pool
    pub fn single_pool(&self) -> bool {
        if self.0 != 0 {
            (self.0 & (self.0 - 1)) == 0
        } else {
            false
        }
    }

    pub fn intersect(&self, other: &Self) -> Self {
        PoolMask(self.0 & other.0)
    }

    pub fn union(&self, other: &Self) -> Self {
        PoolMask(self.0 | other.0)
    }

    pub fn from_address(address: &str) -> Result<Self> {
        let address = ZcashAddress::try_from_encoded(address).map_err(anyhow::Error::from)?;
        let mut pool_mask = 0u8;
        if address.can_receive_as(PoolType::Transparent) {
            pool_mask |= 1;
        }
        if address.can_receive_as(PoolType::Shielded(zcash_protocol::ShieldedProtocol::Sapling)) {
            pool_mask |= 2;
        }
        if address.can_receive_as(PoolType::Shielded(zcash_protocol::ShieldedProtocol::Orchard)) {
            pool_mask |= 4;
        }
        Ok(PoolMask(pool_mask))
    }

    pub fn trim_transparent(self) -> Result<Self> {
        if self.0 == 0 {
            return Err(Error::InvalidPoolMask);
        }
        // if the mask only contains the transparent pool, return it
        if self.0 == 1 {
            return Ok(PoolMask(1));
        }
        // otherwise, mask out the transparent pool
        let masked = self.0 & 0b110;
        Ok(PoolMask(masked))
    }
}

impl From<Option<u8>> for PoolMask {
    fn from(value: Option<u8>) -> Self {
        let p = match value {
            Some(p) => 1 << p,
            None => 0,
        };
        PoolMask(p)
    }
}

