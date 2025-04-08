use self::error::Result;
use pool::PoolMask;

pub mod error;
pub mod fee;
pub mod plan;
pub mod pool;
pub mod prepare;

pub struct Recipient {
    pub address: String,
    pub amount: u64,
    pub pools: Option<u8>,
    pub user_memo: Option<String>,
    pub memo_bytes: Option<Vec<u8>>,
}

pub struct RecipientState {
    pub recipient: Recipient,
    pub remaining: u64,
    pub pool_mask: PoolMask,
}

impl RecipientState {
    pub fn new(recipient: Recipient) -> Result<Self> {
        let amount = recipient.amount;
        let pool_mask = PoolMask::from_address(&recipient.address)?;
        Ok(Self {
            recipient,
            remaining: amount,
            pool_mask,
        })
    }

    pub fn to_inner(self) -> Recipient {
        self.recipient
    }
}
