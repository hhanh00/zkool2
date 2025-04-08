use std::str::FromStr as _;

use anyhow::Result;
use bip32::{PrivateKey, PublicKey as _};
use orchard::keys::FullViewingKey;
use ripemd::{Digest as _, Ripemd160};
use sha2::Sha256;
use zcash_keys::{
    encoding::AddressCodec,
    keys::{UnifiedAddressRequest, UnifiedFullViewingKey, UnifiedSpendingKey},
};
use zcash_primitives::{legacy::TransparentAddress, zip32::AccountId};
use zcash_protocol::consensus::NetworkConstants;
use zcash_transparent::keys::{AccountPrivKey, AccountPubKey, NonHardenedChildIndex, TransparentKeyScope};

use crate::{
    bip38,
    db::{
        init_account_orchard, init_account_sapling, init_account_transparent,
        store_account_orchard_sk, store_account_orchard_vk, store_account_sapling_sk,
        store_account_sapling_vk, store_account_seed, store_account_transparent_addr,
        store_account_transparent_sk, store_account_transparent_vk, update_dindex,
    },
    get_coin,
};

pub async fn put_account_seed(phrase: &str, aindex: u32) -> Result<u32> {
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

    store_account_seed(c.get_pool(), c.account, &phrase, aindex).await?;
    init_account_transparent(c.get_pool(), c.account).await?;
    let txsk = usk.transparent();
    store_account_transparent_sk(c.get_pool(), c.account, txsk).await?;
    let tvk = &txsk.to_account_pubkey();
    store_account_transparent_vk(c.get_pool(), c.account, tvk).await?;
    let tsk = derive_transparent_sk(txsk, dindex)?;
    let taddr = ua.transparent().unwrap();
    store_account_transparent_addr(
        c.get_pool(),
        c.account,
        0,
        dindex,
        Some(&tsk),
        &taddr.encode(&c.network),
    )
    .await?;
    init_account_sapling(c.get_pool(), c.account).await?;
    let sxsk = usk.sapling();
    store_account_sapling_sk(c.get_pool(), c.account, sxsk).await?;
    let sxvk = sxsk.to_diversifiable_full_viewing_key();
    store_account_sapling_vk(c.get_pool(), c.account, &sxvk).await?;
    init_account_orchard(c.get_pool(), c.account).await?;
    let oxsk = usk.orchard();
    store_account_orchard_sk(c.get_pool(), c.account, oxsk).await?;
    let oxvk = FullViewingKey::from(oxsk);
    store_account_orchard_vk(c.get_pool(), c.account, &oxvk).await?;

    update_dindex(c.get_pool(), c.account, dindex, true).await?;

    Ok(aindex)
}

pub async fn put_account_sapling_secret(esk: &str) -> Result<u32> {
    let c = get_coin!();
    let network = c.network;

    let xsk = zcash_keys::encoding::decode_extended_spending_key(
        network.hrp_sapling_extended_spending_key(),
        esk,
    )?;
    init_account_sapling(c.get_pool(), c.account).await?;
    store_account_sapling_sk(c.get_pool(), c.account, &xsk).await?;
    let xvk = xsk.to_diversifiable_full_viewing_key();
    store_account_sapling_vk(c.get_pool(), c.account, &xvk).await?;

    Ok(0)
}

pub async fn put_account_sapling_viewing(evk: &str) -> Result<u32> {
    let c = get_coin!();
    let network = c.network;

    let xvk = zcash_keys::encoding::decode_extended_full_viewing_key(
        network.hrp_sapling_extended_full_viewing_key(),
        evk,
    )?
    .to_diversifiable_full_viewing_key();
    init_account_sapling(c.get_pool(), c.account).await?;
    store_account_sapling_vk(c.get_pool(), c.account, &xvk).await?;

    Ok(0)
}

pub async fn put_account_unified_viewing(uvk: &str) -> Result<u32> {
    let c = get_coin!();
    let network = c.network;

    let uvk =
        UnifiedFullViewingKey::decode(&network, uvk).map_err(|_| anyhow::anyhow!("Invalid Key"))?;
    let (ua, di) = uvk.default_address(UnifiedAddressRequest::AllAvailableKeys)?;
    let dindex: u32 = di.try_into()?;

    if let Some(tvk) = uvk.transparent() {
        init_account_transparent(c.get_pool(), c.account).await?;
        store_account_transparent_vk(c.get_pool(), c.account, tvk).await?;
        let tpk = tvk
            .derive_address_pubkey(
                TransparentKeyScope::EXTERNAL,
                NonHardenedChildIndex::from_index(dindex).unwrap(),
            )
            .unwrap();
        let address = ua.transparent().unwrap();
        store_account_transparent_addr(
            c.get_pool(),
            c.account,
            0,
            dindex,
            None,
            &address.encode(&network),
        )
        .await?;
    }
    if let Some(svk) = uvk.sapling() {
        init_account_sapling(c.get_pool(), c.account).await?;
        store_account_sapling_vk(c.get_pool(), c.account, svk).await?;
    }
    if let Some(ovk) = uvk.orchard() {
        init_account_orchard(c.get_pool(), c.account).await?;
        store_account_orchard_vk(c.get_pool(), c.account, ovk).await?;
    }
    update_dindex(c.get_pool(), c.account, dindex, true).await?;

    Ok(0)
}

pub async fn put_account_transparent_secret(sk: &str) -> Result<u32> {
    let c = get_coin!();
    let network = c.network;

    let sk = bip38::import_tsk(sk)?;
    init_account_transparent(c.get_pool(), c.account).await?;

    let secp = secp256k1::Secp256k1::new();

    let pubkey = sk.public_key(&secp);
    let pubkey = pubkey.to_bytes();
    let ta = TransparentAddress::PublicKeyHash(
        *ripemd::Ripemd160::digest(Sha256::digest(&pubkey)).as_ref(),
    );
    let address = ta.encode(&network);
    store_account_transparent_addr(c.get_pool(), c.account, 0, 0, Some(&sk.to_bytes()), &address).await?;

    Ok(0)
}

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
