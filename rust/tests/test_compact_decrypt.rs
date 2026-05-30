// Test: decrypt orchard notes from compact ciphertext using zkool's try_orchard_decrypt
use zcash_primitives::transaction::Transaction;
use zcash_protocol::consensus::{BranchId, RegtestNetwork};
use zcash_keys::keys::UnifiedSpendingKey;
use zip32::AccountId;
use bip39::Mnemonic;

// Simulate the compact block data that LWD provides
struct CompactOrchardAction {
    nullifier: Vec<u8>,
    cmx: Vec<u8>,
    ephemeral_key: Vec<u8>,
    ciphertext: Vec<u8>, // compact: 84 bytes for ZSA
}

#[test]
fn test_decrypt_compact() {
    let tx_hex = std::fs::read_to_string("tests/shield_tx.hex").unwrap().trim().to_string();
    let tx_bytes = hex::decode(tx_hex).unwrap();

    let tx = Transaction::read(&mut tx_bytes.as_slice(), BranchId::Nu7).unwrap();
    let tx_data = tx.into_data();

    // Derive account 2 keys
    let mnemonic = Mnemonic::parse("burger voice warrior danger satoshi you solid atom elite alcohol category layer able debate culture talk tissue language hip surge fiction paddle stove voyage").unwrap();
    let seed = mnemonic.to_seed("");
    let network = RegtestNetwork;
    let usk = UnifiedSpendingKey::from_seed(&network, &seed, AccountId::try_from(0).unwrap()).unwrap();
    let ufvk = usk.to_unified_full_viewing_key();
    let ofvk = ufvk.orchard().unwrap();

    use zcash_primitives::transaction::OrchardBundle;
    let bundle = match tx_data.orchard_bundle().unwrap() {
        OrchardBundle::OrchardZSA(b) => b,
        _ => panic!("Expected ZSA"),
    };

    let txid_bytes = tx.txid().as_ref().to_vec();

    for scope in [0u8, 1u8] {
        for (action_idx, action) in bundle.actions().iter().enumerate() {
            let full_ct = action.encrypted_note().enc_ciphertext.as_ref();
            // LWD provides only 84 bytes (compact) for ZSA
            let compact_ct = &full_ct[..84];

            println!("\nAction {} scope {}: full_ct={} compact_ct={}",
                action_idx, scope, full_ct.len(), compact_ct.len());

            let ca = CompactOrchardAction {
                nullifier: action.nullifier().to_bytes().to_vec(),
                cmx: action.cmx().to_bytes().to_vec(),
                ephemeral_key: action.encrypted_note().epk_bytes.to_vec(),
                ciphertext: compact_ct.to_vec(),
            };

            // Call zkool's actual decryptor
            let result = rlz::warp::decrypter::try_orchard_decrypt(
                &zcash_trees::network::Network::Regtest,
                2, // account 2
                scope,
                &ofvk.to_ivk(if scope == 0 { orchard::keys::Scope::External } else { orchard::keys::Scope::Internal }),
                1174, // approximate height
                1,    // ivtx
                action_idx as u32,
                &ca,
            );

            match result {
                Ok(Some((note, dbn))) => {
                    println!("  ✅ DECRYPTED: value={} zats, cmx={}",
                        note.value().inner(), hex::encode(&dbn.cmx));
                }
                Ok(None) => {
                    println!("  ❌ No match");
                }
                Err(e) => {
                    println!("  ❌ Error: {:?}", e);
                }
            }
        }
    }
}

// Implement the conversion for CompactOrchardAction
impl<'a> From<&'a CompactOrchardAction> for rlz::lwd::CompactOrchardAction {
    fn from(ca: &'a CompactOrchardAction) -> Self {
        rlz::lwd::CompactOrchardAction {
            nullifier: ca.nullifier.clone(),
            cmx: ca.cmx.clone(),
            ephemeral_key: ca.ephemeral_key.clone(),
            ciphertext: ca.ciphertext.clone(),
        }
    }
}
