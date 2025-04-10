use self::error::Result;
use pool::PoolMask;

pub mod error;
pub mod fee;
pub mod plan;
pub mod pool;
pub mod prepare;

#[derive(Clone, Default, Debug)]
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

    pub fn for_fee(pool: u8, amount: u64) -> Self {
        Self {
            recipient: Recipient {
                amount,
                ..Recipient::default()
            },
            remaining: amount,
            pool_mask: PoolMask::from_pool(pool),
        }
    }

    pub fn to_inner(self) -> Recipient {
        self.recipient
    }
}

#[derive(Clone, Debug)]
pub struct InputNote {
    pub id: u32,
    pub amount: u64,
    pub remaining: u64,
    pub pool: u8,
}

impl InputNote {
    pub fn is_used(&self) -> bool {
        self.remaining != self.amount
    }
}

pub struct TxPlan {
    pub inputs: Vec<TxPlanIn>,
    pub outputs: Vec<TxPlanOut>,
    pub fee: u64,
    pub change: u64,
    pub change_pool: u8,
}

pub struct TxPlanIn {
    pub pool: u8,
    pub amount: u64,
}

pub struct TxPlanOut {
    pub pool: u8,
    pub amount: u64,
    pub address: String,
}
