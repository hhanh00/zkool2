pub mod error;
pub mod pool;
pub mod plan;
pub mod prepare;

pub struct Recipient {
    pub address: String,
    pub amount: u64,
    pub pools: Option<u8>,
    pub user_memo: Option<String>,
    pub memo_bytes: Option<Vec<u8>>,
}
