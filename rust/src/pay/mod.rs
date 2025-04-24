use crate::{lwd::RawTransaction, Client};

use anyhow::Result;
use pczt::Pczt;
use plan::PcztPackage;
use pool::PoolMask;
use tonic::Request;
use zcash_keys::encoding::AddressCodec as _;
use zcash_primitives::legacy::{Script, TransparentAddress};
use zcash_protocol::consensus::Network;

pub mod error;
pub mod fee;
pub mod plan;
pub mod pool;
pub mod prepare;

#[derive(Clone, Default, Debug)]
pub struct Recipient {
    pub address: String,
    pub amount: u64,
    pub pools: Option<u8>,
    pub user_memo: Option<String>,
    pub memo_bytes: Option<Vec<u8>>,
}

pub struct RecipientState {
    pub recipient: Recipient,
    pub remaining: u64,
    pub pool_mask: PoolMask,
}

impl RecipientState {
    pub fn new(recipient: Recipient) -> Result<Self> {
        let amount = recipient.amount;
        let pool_mask = PoolMask::from_address(&recipient.address)?.trim_transparent()?;
        let pm = pool_mask.0;
        assert!(pm == 1 || pm == 2 || pm == 4 || pm == 6);
        Ok(Self {
            recipient,
            remaining: amount,
            pool_mask,
        })
    }

    pub fn for_fee(pool: u8, amount: u64) -> Self {
        Self {
            recipient: Recipient {
                amount,
                ..Recipient::default()
            },
            remaining: amount,
            pool_mask: PoolMask::from_pool(pool),
        }
    }

    pub fn to_inner(self) -> Recipient {
        self.recipient
    }
}

#[derive(Clone, Debug)]
pub struct InputNote {
    pub id: u32,
    pub amount: u64,
    pub remaining: u64,
    pub pool: u8,
}

impl InputNote {
    pub fn is_used(&self) -> bool {
        self.remaining != self.amount
    }
}

pub struct SerializedPCZT {
    pub height: u32,
    pub signed: bool,
    pub data: Vec<u8>,
}

pub struct TxPlan {
    pub height: u32,
    pub inputs: Vec<TxPlanIn>,
    pub outputs: Vec<TxPlanOut>,
    pub fee: u64,
    pub change: u64,
    pub change_pool: u8,
}

impl TxPlan {
    pub fn from_package(network: &Network, package: &PcztPackage) -> Result<Self> {
        let mut inputs = vec![];
        let mut outputs = vec![];

        let pczt = Pczt::parse(&package.pczt).map_err(|e| e.into())?;
        let bundle = pczt.transparent();
        for i in bundle.inputs().iter() {
            inputs.push(TxPlanIn {
                pool: 0,
                amount: *i.value(),
            });
        }
        for o in bundle.outputs().iter() {
            let script_pubkey = Script(o.script_pubkey().to_vec());
            outputs.push(TxPlanOut {
                pool: 0,
                amount: *o.value(),
                address: script_pubkey.address().unwrap().encode(network),
            });
        }

        let bundle = pczt.sapling();
        let mut net = 0;
        for i in bundle.spends().iter() {
            let amount = u64::from_be_bytes(i.proprietary()["value"].try_into().unwrap());
            inputs.push(TxPlanIn {
                pool: 1,
                amount,
            });
            net += amount as i64;
        }
        for o in bundle.outputs().iter() {
            let amount = u64::from_be_bytes(o.proprietary()["value"].try_into().unwrap());
            outputs.push(TxPlanOut {
                pool: 1,
                amount,
                address: o.address().unwrap().encode(network),
            });
            net -= amount as i64;
        }


        TxPlan {
            height: package.height,
            inputs,
            outputs,
            fee: todo!(),
            change: todo!(),
            change_pool: todo!(),
        };
        todo!()
    }
}

pub struct TxPlanIn {
    pub pool: u8,
    pub amount: u64,
}

pub struct TxPlanOut {
    pub pool: u8,
    pub amount: u64,
    pub address: String,
}

pub async fn send(client: &mut Client, height: u32, data: &[u8]) -> Result<String> {
    let rep = client
        .send_transaction(Request::new(RawTransaction {
            height: height as u64,
            data: data.to_vec(),
        }))
        .await?
        .into_inner();
    if rep.error_code != 0 {
        return Err(anyhow::anyhow!(rep.error_message));
    }
    Ok(rep.error_message)
}
