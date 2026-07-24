//! Replay PCZT orchard proving from a saved file.
//! Usage: cargo run --bin pczt_replay -- <input.pczt> <output.pczt>

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <input.pczt> <output.pczt>", args[0]);
        std::process::exit(1);
    }

    let bytes = std::fs::read(&args[1]).expect("read input");
    let pczt = pczt::Pczt::parse(&bytes).expect("parse PCZT");

    let is_zsa = zcash_protocol::consensus::BranchId::try_from(*pczt.global().consensus_branch_id())
        .map(|b| b == zcash_protocol::consensus::BranchId::Nu7)
        .unwrap_or(false);

    let orchard_pk = if is_zsa {
        orchard::circuit::ProvingKey::build_zsa()
    } else {
        orchard::circuit::ProvingKey::build(orchard::circuit::OrchardCircuitVersion::FixedPostNu6_2)
    };

    let prover = pczt::roles::prover::Prover::new(pczt)
        .create_orchard_proof(&orchard_pk)
        .expect("orchard proof");
    let pczt = prover.finish();

    let out = pczt.serialize().expect("serialize");
    std::fs::write(&args[2], &out).expect("write output");
    eprintln!("Wrote {} bytes to {}", out.len(), args[2]);
}
