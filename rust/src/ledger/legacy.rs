// Legacy Ledger App (Non Shielded)

use std::io::{Cursor, Read, Write};

use anyhow::Result;
use byteorder::{ReadBytesExt, LE};
use byteorder::WriteBytesExt;
use pczt::Pczt;
use sqlx::SqliteConnection;
use zcash_keys::encoding::AddressCodec;
use zcash_primitives::legacy::TransparentAddress;
use zcash_primitives::transaction::Transaction;

use crate::coin::Network;
use crate::db::LEDGER_CODE;
use crate::ledger::{APDUCommand, Device};
use crate::Client;

pub async fn derive_hw_transparent_address<D: Device>(
    device: &D,
    network: &Network,
    hw_code: u32,
    aindex: u32,
    scope: u32,
    dindex: u32,
) -> Result<(Vec<u8>, TransparentAddress)> {
    assert_eq!(hw_code, LEDGER_CODE);

    let mut params = vec![];
    // 5 parts, begins with m/44'/133'
    params.extend_from_slice(&hex::decode("058000002C80000085").unwrap());
    // account'
    params.extend_from_slice(&(aindex | 0x80000000).to_be_bytes());
    // external
    params.extend_from_slice(&(scope).to_be_bytes());
    // dindex
    params.extend_from_slice(&(dindex).to_be_bytes());

    let get_pk = APDUCommand {
        cla: 0xE0,
        ins: 0x40,
        p1: 0,
        p2: 0,
        data: params,
    };
    let rep = device.execute(&get_pk).await?;
    if rep.retcode != 0x9000 {
        anyhow::bail!("Ledger error {}", rep.retcode);
    }
    let mut data = Cursor::new(&rep.data);
    let pk_len = data.read_u8()? as usize;
    let mut pk = vec![0u8; pk_len];
    data.read_exact(&mut pk)?;
    let address_len = data.read_u8()? as usize;
    let mut address = vec![0u8; address_len];
    data.read_exact(&mut address)?;
    let address = String::from_utf8(address)?;
    let address = TransparentAddress::decode(network, &address)?;
    Ok((pk, address))
}

pub fn get_trusted_input(tx: &Transaction, index: u32) -> Result<Vec<Vec<u8>>> {
    /*
    - output index - 4 bytes
    - version
    - version group
    - branch id
    - # tin
        - prev txid
        - prev vout
        - redeem script
        - sequence
    - # tout
        - value
        - pubscript
    - # sin/sout/oact
    - if sapling
        - value balance
        - anchor if spend
        - spends
            - cv
            - nf
            - rk
        - outputs
            - cmu
            - epk
            - cenc comp
        - outputs
            - memo
        - outputs
            - cv
            - cenc authd
            - cout
    - if orchard
        - for each action
            - nf
            - cmx
            - epk
            - cenc comp
        - for each action
            - memos
        - for each action
            - cv
            - rk
            - authd
            - cout
        - flags
        - balance
        - anchor
    */

    let mut buffers = vec![];
    let mut buffer = vec![];
    buffer.write_u32::<LE>(index)?;
    buffer.write_u32::<LE>(tx.version().header())?;
    buffer.write_u32::<LE>(tx.version().version_group_id())?;
    buffer.write_u32::<LE>(tx.consensus_branch_id().into())?;
    if let Some(tbundle) = tx.transparent_bundle() {
        buffer.write_u8(tbundle.vin.len() as u8)?; // TODO use compact
        for tin in tbundle.vin.iter() {
            buffer.write_all(tin.prevout().hash())?;
            buffer.write_u32::<LE>(tin.prevout().n())?;
            tin.script_sig().write(&mut buffer)?;
            buffer.write_u32::<LE>(tin.sequence())?;
            buffers.push(std::mem::take(&mut buffer));
            buffer.clear();
        }
        buffer.write_u8(tbundle.vout.len() as u8)?; // TODO use compact
        for tout in tbundle.vout.iter() {
            buffer.write_u64::<LE>(tout.value().into_u64())?;
            tout.script_pubkey().write(&mut buffer)?;
            buffers.push(std::mem::take(&mut buffer));
            buffer.clear();
        }
    } else {
        buffer.write_u16::<LE>(0)?;
        buffers.push(std::mem::take(&mut buffer));
        buffer.clear();
    }
    let (sin, sout) = tx
        .sapling_bundle()
        .map(|b| (b.shielded_spends().len(), b.shielded_outputs().len()))
        .unwrap_or_default();
    let oact = tx
        .orchard_bundle()
        .map(|b| b.actions().len())
        .unwrap_or_default();
    buffer.write_u8(sin as u8)?; // TODO use compact
    buffer.write_u8(sout as u8)?; // TODO use compact
    buffer.write_u8(oact as u8)?; // TODO use compact
    buffers.push(std::mem::take(&mut buffer));
    buffer.clear();

    if let Some(sbundle) = tx.sapling_bundle() {
        buffer.write_i64::<LE>(sbundle.value_balance().into())?;
        if sin > 0 {
            buffer.write_all(&sbundle.shielded_spends()[0].anchor().to_bytes())?;
        }
        buffers.push(std::mem::take(&mut buffer));
        buffer.clear();
        for sin in sbundle.shielded_spends().iter() {
            buffer.write_all(&sin.cv().to_bytes())?;
            buffer.write_all(&sin.nullifier().to_vec())?;
            let rk: [u8; 32] = (*sin.rk()).into();
            buffer.write_all(&rk)?;
            buffers.push(std::mem::take(&mut buffer));
            buffer.clear();
        }
        for sout in sbundle.shielded_outputs() {
            buffer.write_all(&sout.cmu().to_bytes())?;
            buffer.write_all(&sout.ephemeral_key().0)?;
            buffer.write_all(&sout.enc_ciphertext()[0..52])?;
            buffers.push(std::mem::take(&mut buffer));
            buffer.clear();
        }
        for sout in sbundle.shielded_outputs() {
            for i in 0..4 {
                buffer.write_all(&sout.enc_ciphertext()[52 + i * 128..52 + (i + 1) * 128])?;
                buffers.push(std::mem::take(&mut buffer));
                buffer.clear();
            }
        }
        for sout in sbundle.shielded_outputs() {
            buffer.write_all(&sout.cv().to_bytes())?;
            buffer.write_all(&sout.enc_ciphertext()[52 + 512..])?;
            buffer.write_all(sout.out_ciphertext())?;
            buffers.push(std::mem::take(&mut buffer));
            buffer.clear();
        }
    }

    if let Some(obundle) = tx.orchard_bundle() {
        for a in obundle.actions().iter() {
            buffer.write_all(&a.nullifier().to_bytes())?;
            buffer.write_all(&a.cmx().to_bytes())?;
            buffer.write_all(&a.encrypted_note().epk_bytes)?;
            buffer.write_all(&a.encrypted_note().enc_ciphertext[..52])?;
            buffers.push(std::mem::take(&mut buffer));
            buffer.clear();
        }
        for a in obundle.actions().iter() {
            for i in 0..4 {
                buffer.write_all(
                    &a.encrypted_note().enc_ciphertext[52 + i * 128..52 + (i + 1) * 128],
                )?;
                buffers.push(std::mem::take(&mut buffer));
                buffer.clear();
            }
        }
        for a in obundle.actions().iter() {
            buffer.write_all(&a.cv_net().to_bytes())?;
            let rk: [u8; 32] = (a.rk()).into();
            buffer.write_all(&rk)?;
            buffer.write_all(&a.encrypted_note().enc_ciphertext[52 + 512..])?;
            buffer.write_all(&a.encrypted_note().out_ciphertext)?;
            buffers.push(std::mem::take(&mut buffer));
            buffer.clear();
        }

        buffer.write_u8(obundle.flags().to_byte())?;
        buffer.write_i64::<LE>(obundle.value_balance().into())?;
        buffer.write_all(&obundle.anchor().to_bytes())?;
        buffers.push(std::mem::take(&mut buffer));
        buffer.clear();
    }
    buffer.write_u32::<LE>(tx.lock_time())?;
    buffer.write_u8(4)?; // len of extra data
    buffer.write_u32::<LE>(tx.expiry_height().into())?;
    buffers.push(std::mem::take(&mut buffer));
    buffer.clear();

    Ok(buffers)
}

pub async fn sign_transaction(
    _network: &Network,
    _connection: &SqliteConnection,
    _account: u32,
    _client: &mut Client,
    _pczt: &Pczt,
) -> Result<()> {
    /*
    How to sign

    1. get trusted inputs
       1. send previous transactions to ins 0x42 and get the txid
    2. use 0x44
       1. send transaction up to outputs replacing tins by the trusted inputs
          1. (p1, p2) = (00, 05) then (80, 80)
       2. add prev pubscripts (should be our wallet addresses) + sequence
    3. use 0x4a
       1. set change output, p1 = FF
       2. set other outputs too
    4. send e04800000b 0000000000000100000000
    5. use 0x44 again but with 1 input at a time
    6. sign with 0x48 + path
    */

    todo!()
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read};

    use byteorder::ReadBytesExt;

    use crate::{
        coin::{Coin, Network, ServerType},
        ledger::{
            APDUCommand, LEDGER_ZEMU
        },
    };

    use super::*;

    #[tokio::test]
    async fn test_get_trusted_inputs() -> anyhow::Result<()> {
        let mut txid =
            hex::decode("339df469e5b5e0259e214a1c653cdcb28c2c2e9664c5b5df5d518974950c36e1")?;
        txid.reverse();
        let coin = Coin::new(ServerType::Lwd, "https://zec.rocks", false, "test.db", None).await?;
        let mut client = coin.client().await?;
        let (_, tx) = client.transaction(&Network::Main, &txid).await?;
        let payload = super::get_trusted_input(&tx, 0)?;

        LEDGER_ZEMU
            .lock()
            .await
            .clone()
            .unwrap()
            .long_execute(
                &APDUCommand {
                    cla: 0xE0,
                    ins: 0x42,
                    p1: 0,
                    p2: 0,
                    data: vec![],
                },
                &payload,
            )
            .await?;
        Ok(())
    }

    // This is for the Transparent Only Ledger app
    #[allow(dead_code)]
    async fn get_pk() -> anyhow::Result<()> {
        let get_pk = APDUCommand {
            cla: 0xE0,
            ins: 0x40,
            p1: 0,
            p2: 0,
            data: vec![],
        };
        let rep = LEDGER_ZEMU
            .lock()
            .await
            .clone()
            .unwrap()
            .long_execute(
                &get_pk,
                &[hex::decode("058000002C80000085800000000000000000000000").unwrap()],
            )
            .await?;
        println!("{}", rep.retcode);
        let mut data = Cursor::new(&rep.data);
        let pk_len = data.read_u8()? as usize;
        let mut pk = vec![0u8; pk_len];
        data.read_exact(&mut pk)?;
        let address_len = data.read_u8()? as usize;
        let mut address = vec![0u8; address_len];
        data.read_exact(&mut address)?;
        let address = String::from_utf8(address)?;
        println!("{address}");
        Ok(())
    }
}

