use anyhow::Result;
use rand_core::OsRng;
use rlz::ledger::{builder::z2z, LEDGER};
use zcash_proofs::prover::LocalTxProver;

#[tokio::main]
pub async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let account = args.get(1).expect("<account>");
    let stage = args.get(2).expect("<stage>");
    let ledger = LEDGER.lock().await;
    let prover = LocalTxProver::with_default_location().unwrap();
    let rng = OsRng;
    z2z(stage.parse::<u8>()?, account.parse::<u32>()?, &ledger, &prover, rng).await?;

    Ok(())
}
