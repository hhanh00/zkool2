use anyhow::Result;
use fpdec::Decimal;
use zcash_protocol::memo::Memo;
use zip321::TransactionRequest;

use crate::pay::Recipient;

pub fn to_zec(amount: u64) -> String {
    let zats = fpdec::Decimal::from(amount);
    let zec: Decimal = zats / 100_000_000;
    zec.to_string()
}

pub fn parse_payment_uri(uri: &str) -> Result<Vec<Recipient>> {
    let uri = uri.trim();
    if uri.is_empty() {
        return Ok(vec![]);
    }

    let uri = TransactionRequest::from_uri(uri)?;
    let recipients: Result<Vec<_>> = uri
        .payments()
        .iter()
        .map(|(_, payment)| {
            let address = payment.recipient_address().to_string();
            let amount = payment.amount().into_u64();
            let memo = payment.memo();
            let memo_text = memo
                .map(|m| {
                    let m = Memo::try_from(m)?;
                    Ok::<_, anyhow::Error>(match m {
                        Memo::Empty => Some("".to_string()),
                        Memo::Text(text_memo) => Some(text_memo.to_string()),
                        _ => None,
                    })
                })
                .transpose()?;
            let memo_text = memo_text.flatten();

            let recipient = Recipient {
                address: address.to_string(),
                amount,
                memo_bytes: memo.map(|m| m.clone().into_bytes().to_vec()),
                user_memo: memo_text,
                ..Recipient::default()
            };
            Ok(recipient)
        })
        .collect::<Result<Vec<_>>>();

    recipients
}
