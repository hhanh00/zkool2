use std::str::FromStr as _;

use anyhow::Result;
use bip32::PublicKey as _;
use orchard::keys::FullViewingKey;
use ripemd::{Digest as _, Ripemd160};
use sha2::Sha256;
use zcash_keys::{
    encoding::AddressCodec,
    keys::{UnifiedAddressRequest, UnifiedFullViewingKey, UnifiedSpendingKey},
};
use zcash_primitives::{legacy::TransparentAddress, zip32::AccountId};
use zcash_protocol::consensus::NetworkConstants;
use zcash_transparent::keys::{
    AccountPubKey, NonHardenedChildIndex, TransparentKeyScope
};

use crate::{bip38, db::{init_account_orchard, init_account_sapling, init_account_transparent, store_account_orchard_sk, store_account_orchard_vk, store_account_sapling_sk, store_account_sapling_vk, store_account_seed, store_account_transparent_addr, store_account_transparent_sk, store_account_transparent_vk, update_dindex}, get_coin, setup};

pub fn put_account_seed(id: u32, phrase: &str, aindex: u32) -> Result<u32> {
    setup!(id);

    let c = get_coin!();

    let seed_phrase = bip39::Mnemonic::from_str(phrase)?;
    let seed = seed_phrase.to_seed("");
    let usk =
        UnifiedSpendingKey::from_seed(&c.network, &seed, AccountId::try_from(aindex).unwrap())?;
    let uvk = usk.to_unified_full_viewing_key();
    println!("{}", uvk.encode(&c.network));
    let (ua, di) = uvk.default_address(UnifiedAddressRequest::AllAvailableKeys)?;
    let dindex: u32 = di.try_into()?;
    println!("dindex: {dindex}");

    store_account_seed(&phrase, aindex)?;
    init_account_transparent()?;
    let tsk = usk.transparent();
    store_account_transparent_sk(tsk)?;
    let tvk = &tsk.to_account_pubkey();
    store_account_transparent_vk(tvk)?;
    let taddr = ua.transparent().unwrap();
    store_account_transparent_addr(
        0,
        dindex,
        &tsk.to_bytes(),
        &taddr.encode(&c.network),
    )?;
    init_account_sapling()?;
    let sxsk = usk.sapling();
    store_account_sapling_sk(sxsk)?;
    let sxvk = sxsk.to_diversifiable_full_viewing_key();
    store_account_sapling_vk(&sxvk)?;
    init_account_orchard()?;
    let oxsk = usk.orchard();
    store_account_orchard_sk(oxsk)?;
    let oxvk = FullViewingKey::from(oxsk);
    store_account_orchard_vk(&oxvk)?;

    update_dindex(dindex, true)?;

    Ok(id)
}

pub fn put_account_sapling_secret(id: u32, esk: &str) -> Result<u32> {
    setup!(id);

    let c = get_coin!();
    let network = c.network;

    let xsk = zcash_keys::encoding::decode_extended_spending_key(
        network.hrp_sapling_extended_spending_key(),
        esk,
    )?;
    init_account_sapling()?;
    store_account_sapling_sk(&xsk)?;
    let xvk = xsk.to_diversifiable_full_viewing_key();
    store_account_sapling_vk(&xvk)?;

    Ok(id)
}

pub fn put_account_sapling_viewing(id: u32, evk: &str) -> Result<u32> {
    setup!(id);

    let c = get_coin!();
    let network = c.network;

    let xvk = zcash_keys::encoding::decode_extended_full_viewing_key(
        network.hrp_sapling_extended_full_viewing_key(),
        evk,
    )?
    .to_diversifiable_full_viewing_key();
    init_account_sapling()?;
    store_account_sapling_vk(&xvk)?;

    Ok(id)
}

pub fn put_account_unified_viewing(id: u32, uvk: &str) -> Result<u32> {
    setup!(id);

    let c = get_coin!();
    let network = c.network;

    let uvk = UnifiedFullViewingKey::decode(&network, uvk).map_err(|_| anyhow::anyhow!("Invalid Key"))?;
    let (ua, di) = uvk.default_address(UnifiedAddressRequest::AllAvailableKeys)?;
    let dindex: u32 = di.try_into()?;

    if let Some(tvk) = uvk.transparent() {
        init_account_transparent()?;
        store_account_transparent_vk(tvk)?;
        let tpk = tvk
            .derive_address_pubkey(
                TransparentKeyScope::EXTERNAL,
                NonHardenedChildIndex::from_index(dindex).unwrap(),
            )
            .unwrap();
        let address = ua.transparent().unwrap();
        store_account_transparent_addr(0, dindex, &tpk.serialize(), &address.encode(&network))?;
    }
    if let Some(svk) = uvk.sapling() {
        init_account_sapling()?;
        store_account_sapling_vk(svk)?;
    }
    if let Some(ovk) = uvk.orchard() {
        init_account_orchard()?;
        store_account_orchard_vk(ovk)?;
    }
    update_dindex(dindex, true)?;

    Ok(id)
}

pub fn put_account_transparent_secret(id: u32, sk: &str) -> Result<u32> {
    setup!(id);
    let c = get_coin!();
    let network = c.network;

    let sk = bip38::import_tsk(sk)?;
    init_account_transparent()?;

    let secp = secp256k1::Secp256k1::new();

    let pubkey = sk.public_key(&secp);
    let pubkey = pubkey.to_bytes();
    let ta = TransparentAddress::PublicKeyHash(
        *ripemd::Ripemd160::digest(Sha256::digest(&pubkey)).as_ref(),
    );
    let address = ta.encode(&network);
    store_account_transparent_addr(0, 0, &pubkey, &address)?;

    Ok(id)
}

pub fn derive_transparent_address(
    tvk: &AccountPubKey,
    scope: u32,
    dindex: u32,
) -> Result<([u8; 33], TransparentAddress)> {
    let sindex = TransparentKeyScope::custom(scope).unwrap();
    let tpk = tvk
        .derive_address_pubkey(sindex, NonHardenedChildIndex::from_index(dindex).unwrap())
        .unwrap()
        .serialize();
    let pkh: [u8; 20] = Ripemd160::digest(&Sha256::digest(&tpk)).into();
    let addr = TransparentAddress::PublicKeyHash(pkh);
    Ok((tpk, addr))
}
