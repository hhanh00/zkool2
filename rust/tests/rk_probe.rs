//! Temporary probe: does the Ironwood PCZT spend's embedded rk equal
//! `ak.randomize(alpha)` (the zkp1 recomputation)? Reproduces the
//! "delegation proof result rk does not match stored PCZT data" failure
//! without any database.

use orchard::builder::{Builder, BundleType};
use orchard::bundle::{BundleVersion, TxVersion as OrchardTxVersion};
use orchard::keys::{FullViewingKey, Scope, SpendValidatingKey};
use orchard::note::{AssetBase, RandomSeed, Rho};
use orchard::value::NoteValue;
use orchard::{Note, NoteVersion};
use rand_core::OsRng;
use zcash_keys::keys::UnifiedSpendingKey;
use zip32::AccountId;

use zcash_trees::network::Network as WalletNetwork;

#[test]
fn ironwood_pczt_spend_rk_matches_randomize() {
    let network = WalletNetwork::Main;
    let usk = UnifiedSpendingKey::from_seed(&network, &[0x42; 32], AccountId::try_from(0).unwrap())
        .unwrap();
    let ufvk = usk.to_unified_full_viewing_key();
    let fvk = FullViewingKey::from(usk.orchard());

    let recipient = fvk.address_at(0u32, Scope::External);
    let rho = Rho::from_bytes(&[9; 32]).unwrap();
    let rseed = RandomSeed::from_bytes([7; 32], &rho).unwrap();
    let note = Note::from_parts(
        recipient,
        NoteValue::from_raw(1),
        AssetBase::zatoshi(),
        rho,
        rseed,
        NoteVersion::V3,
    )
    .unwrap();

    let bundle_version = BundleVersion::ironwood_v3();
    let mut builder = Builder::new_with_anchor_deferred(
        BundleType::UNPADDED,
        bundle_version,
        bundle_version.default_flags(),
        OrchardTxVersion::V6,
    )
    .expect("builder");
    builder
        .add_spend_unwitnessed(fvk.clone(), note)
        .expect("spend");
    builder
        .add_output(None, recipient, NoteValue::ZERO, [0u8; 512])
        .expect("output");
    let (pczt_bundle, _meta) = builder.build_for_pczt(&mut OsRng).expect("build");

    let action = &pczt_bundle.actions()[0];
    let rk_from_pczt: [u8; 32] = action.spend().rk().into();
    let alpha = action.spend().alpha().expect("alpha");

    let ak: SpendValidatingKey = fvk.clone().into();
    let rk_from_randomize: [u8; 32] = (&ak.randomize(&alpha)).into();

    println!("ufvk orchard fvk : {}", hex::encode(fvk_bytes(&ufvk, &fvk)));
    println!("rk from pczt     : {}", hex::encode(rk_from_pczt));
    println!("rk from randomize: {}", hex::encode(rk_from_randomize));
    assert_eq!(
        rk_from_pczt, rk_from_randomize,
        "Ironwood PCZT rk diverges from ak.randomize(alpha)"
    );
}

fn fvk_bytes(_ufvk: &zcash_keys::keys::UnifiedFullViewingKey, fvk: &FullViewingKey) -> [u8; 96] {
    fvk.to_bytes()
}
