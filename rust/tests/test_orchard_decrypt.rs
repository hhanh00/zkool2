// Test: decrypt orchard notes from V6 shield TX using account 2's keys
use orchard::{
    keys::{FullViewingKey, Scope, PreparedIncomingViewingKey},
    primitives::OrchardDomain,
    flavor::OrchardZSA,
    note::ExtractedNoteCommitment,
};
use zcash_note_encryption::try_note_decryption;
use zcash_primitives::transaction::Transaction;
use zcash_protocol::consensus::{BranchId, RegtestNetwork};
use zcash_keys::keys::UnifiedSpendingKey;
use zip32::AccountId;
use bip39::Mnemonic;

#[test]
fn test_decrypt_orchard_notes() {
    let tx_hex = std::fs::read_to_string("tests/shield_tx.hex").unwrap().trim().to_string();
    let tx_bytes = hex::decode(tx_hex).unwrap();
    println!("TX size: {} bytes", tx_bytes.len());

    let tx = Transaction::read(&mut tx_bytes.as_slice(), BranchId::Nu7)
        .expect("Failed to parse V6 transaction");
    println!("txid: {}", tx.txid());
    let tx_data = tx.into_data();

    // Derive account 2's keys
    let seed_phrase = "burger voice warrior danger satoshi you solid atom elite alcohol category layer able debate culture talk tissue language hip surge fiction paddle stove voyage";
    let mnemonic = Mnemonic::parse(seed_phrase).unwrap();
    let seed = mnemonic.to_seed("");
    let network = RegtestNetwork;

    let usk = UnifiedSpendingKey::from_seed(
        &network,
        &seed,
        AccountId::try_from(0).unwrap(),
    ).expect("Failed to derive USK");
    let ufvk = usk.to_unified_full_viewing_key();
    let ofvk = ufvk.orchard().expect("No orchard key");

    // Print orchard address
    let diversifier = orchard::keys::Diversifier::from_bytes([0u8; 11]);
    let addr = ofvk.address(diversifier, Scope::External);
    let ua = zcash_keys::address::UnifiedAddress::from_receivers(Some(addr), None, None).unwrap();
    println!("Account 2 orchard addr: {}", ua.encode(&network));

    // Get actions
    let obundle = match tx_data.orchard_bundle() {
        Some(b) => b,
        None => { println!("No orchard bundle!"); return; }
    };

    use zcash_primitives::transaction::OrchardBundle;
    let actions = match obundle {
        OrchardBundle::OrchardVanilla(_) => {
            println!("Vanilla bundle — unexpected");
            return;
        }
        OrchardBundle::OrchardZSA(bundle) => {
            println!("ZSA bundle, {} actions", bundle.actions().len());
            bundle.actions().iter().collect::<Vec<_>>()
        }
    };

    let mut found_any = false;
    for scope in [Scope::External, Scope::Internal] {
        let ivk = ofvk.to_ivk(scope);
        let pivk = PreparedIncomingViewingKey::new(&ivk);

        for (i, action) in actions.iter().enumerate() {
            let domain = OrchardDomain::<OrchardZSA>::for_action(action);
            if let Some((note, recipient, memo)) = try_note_decryption(&domain, &pivk, action) {
                found_any = true;
                println!("✅ Action {} scope {:?}: value={}, is_zatoshi={}",
                    i, scope, note.value().inner(), bool::from(note.asset().is_zatoshi()));
                println!("   diversifier: {}", hex::encode(recipient.diversifier().as_array()));
                let cmx = ExtractedNoteCommitment::from(note.commitment());
                println!("   cmx: {}", hex::encode(cmx.to_bytes()));
            }
        }
    }
    if !found_any {
        println!("❌ No notes decrypted for our keys!");
        println!("This means the tx does NOT contain notes for account 2.");
    }
}
