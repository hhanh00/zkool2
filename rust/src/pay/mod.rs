pub mod error;
pub mod prepare;

pub struct Recipient {
    pub address: String,
    pub amount: u64,
    pub pools: u8,
    pub user_memo: String,
    pub memo_bytes: Vec<u8>,
}
