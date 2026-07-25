//! TEMP diagnostic: extract the raw transaction bytes from a saved postsigned PCZT
//! and write them to /tmp/zsa_tx.bin (also reports whether local extract-verify passes).
//! Usage: cargo run --bin dump_tx -- [/tmp/zsa_postsigned.pczt]

use rlz::api::pay::{extract_transaction, PcztPackage};

#[tokio::main]
async fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/zsa_postsigned.pczt".to_string());
    let pczt = std::fs::read(&path).expect("read pczt");
    let pkg = PcztPackage {
        pczt,
        n_spends: [0; 4],
        sapling_indices: vec![],
        orchard_indices: vec![],
        ironwood_indices: vec![],
        can_sign: false,
        can_broadcast: true,
        price: None,
        category: None,
        is_issuance: false,
    };
    match extract_transaction(&pkg).await {
        Ok(tx) => {
            std::fs::write("/tmp/zsa_tx.bin", &tx).unwrap();
            eprintln!("OK: local extract+verify passed; wrote {} bytes to /tmp/zsa_tx.bin", tx.len());
            // Re-parse the emitted bytes with THIS (new) lrz and report structure.
            use zcash_primitives::transaction::{OrchardBundle, Transaction};
            use zcash_protocol::consensus::BranchId;
            match Transaction::read(&tx[..], BranchId::Nu7) {
                Ok(parsed) => {
                    eprintln!("SELF RE-PARSE (new lrz): OK");
                    if let Some(b) = parsed.orchard_bundle() {
                        eprintln!("  bundle_version = {:?}", b.bundle_version());
                        eprintln!("  flags: spends={} outputs={} zsa_enabled={}", b.flags().spends_enabled(), b.flags().outputs_enabled(), b.flags().zsa_enabled());
                        eprintln!("  flag_byte = {:#04x}", b.flag_byte());
                        match b {
                            OrchardBundle::OrchardVanilla(b) => {
                                eprintln!("  orchard actions = {}", b.actions().len());
                                for (i, a) in b.actions().iter().enumerate() {
                                    eprintln!(
                                        "  action[{i}] enc_ciphertext len = {}",
                                        a.encrypted_note().enc_ciphertext.as_ref().len()
                                    );
                                }
                            }
                            OrchardBundle::OrchardZSA(b) => {
                                eprintln!("  ZSA orchard actions = {}", b.actions().len());
                                for (i, a) in b.actions().iter().enumerate() {
                                    eprintln!(
                                        "  action[{i}] enc_ciphertext len = {}",
                                        a.encrypted_note().enc_ciphertext.as_ref().len()
                                    );
                                }
                            }
                        }
                    } else {
                        eprintln!("  no orchard bundle in re-parsed tx!");
                    }
                }
                Err(e) => eprintln!("SELF RE-PARSE (new lrz) FAILED: {e:?}"),
            }
        }
        Err(e) => {
            eprintln!("EXTRACT FAILED (local verify rejected): {e:?}");
            std::process::exit(1);
        }
    }
}
