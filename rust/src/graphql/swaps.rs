use crate::near_intents::SavedSwap;
use bigdecimal::BigDecimal;

// GraphQL IDs and decimal timestamps retain the full SQLite integer range.
#[juniper::graphql_object]
impl SavedSwap {
    fn id_swap(&self) -> juniper::ID {
        self.id_swap.to_string().into()
    }
    fn account(&self) -> juniper::ID {
        self.account.to_string().into()
    }
    fn origin_asset(&self) -> &str {
        &self.origin_asset
    }
    fn destination_asset(&self) -> &str {
        &self.destination_asset
    }
    fn swap_type(&self) -> &str {
        &self.swap_type
    }
    fn amount(&self) -> &str {
        &self.amount
    }
    fn slippage_tolerance(&self) -> i32 {
        self.slippage_tolerance
    }
    fn recipient(&self) -> &str {
        &self.recipient
    }
    fn refund_to(&self) -> &str {
        &self.refund_to
    }
    fn deadline(&self) -> &str {
        &self.deadline
    }
    fn amount_in(&self) -> &str {
        &self.amount_in
    }
    fn amount_out(&self) -> &str {
        &self.amount_out
    }
    fn min_amount_in(&self) -> Option<&str> {
        self.min_amount_in.as_deref()
    }
    fn min_amount_out(&self) -> Option<&str> {
        self.min_amount_out.as_deref()
    }
    fn deposit_address(&self) -> &str {
        &self.deposit_address
    }
    fn deposit_memo(&self) -> Option<&str> {
        self.deposit_memo.as_deref()
    }
    fn deposit_tx_hash(&self) -> Option<&str> {
        self.deposit_tx_hash.as_deref()
    }
    fn deposit_submitted_at(&self) -> Option<BigDecimal> {
        self.deposit_submitted_at.map(BigDecimal::from)
    }
    fn quote_response(&self) -> &str {
        &self.quote_response
    }
    fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }
    fn status_response(&self) -> Option<&str> {
        self.status_response.as_deref()
    }
    fn last_checked_at(&self) -> Option<BigDecimal> {
        self.last_checked_at.map(BigDecimal::from)
    }
    fn completed_at(&self) -> Option<BigDecimal> {
        self.completed_at.map(BigDecimal::from)
    }
    fn created_at(&self) -> BigDecimal {
        self.created_at.into()
    }
    fn updated_at(&self) -> BigDecimal {
        self.updated_at.into()
    }
}
