use anyhow::Result;
use bip32::PrivateKey;
use ripemd::{Digest as _, Ripemd160};
use sha2::Sha256;
use zcash_primitives::legacy::TransparentAddress;
use zcash_transparent::keys::{AccountPrivKey, AccountPubKey, NonHardenedChildIndex, TransparentKeyScope};


pub fn derive_transparent_sk(
    tvk: &AccountPrivKey,
    dindex: u32,
) -> Result<[u8; 32]> {
    let tsk = tvk
        .derive_external_secret_key(NonHardenedChildIndex::from_index(dindex).unwrap())
        .unwrap()
        .to_bytes();
    Ok(tsk)
}

pub fn derive_transparent_address(
    tvk: &AccountPubKey,
    scope: u32,
    dindex: u32,
) -> Result<TransparentAddress> {
    let sindex = TransparentKeyScope::custom(scope).unwrap();
    let tpk = tvk
        .derive_address_pubkey(sindex, NonHardenedChildIndex::from_index(dindex).unwrap())
        .unwrap()
        .serialize();
    let pkh: [u8; 20] = Ripemd160::digest(&Sha256::digest(&tpk)).into();
    let addr = TransparentAddress::PublicKeyHash(pkh);
    Ok(addr)
}
