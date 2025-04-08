use self::error::Result;
use pool::PoolMask;

pub mod error;
pub mod fee;
pub mod plan;
pub mod pool;
pub mod prepare;

#[derive(Clone, Debug)]
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
        let pool_mask = PoolMask::from_address(&recipient.address)?
            .trim_transparent()?;
        let pm = pool_mask.0;
        assert!(pm == 1 || pm == 2 || pm == 4 || pm == 6);
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
