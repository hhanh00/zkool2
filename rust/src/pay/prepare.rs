use fpdec::Decimal;

pub fn to_zec(amount: u64) -> String {
    let zats = fpdec::Decimal::from(amount);
    let zec: Decimal = zats / 100_000_000;
    zec.to_string()
}
