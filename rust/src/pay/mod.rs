use crate::{api::pay::PcztPackage, Client};

use anyhow::Result;
use pczt::{roles::verifier::Verifier, Pczt};
use pool::PoolMask;
use tracing::{info, span, Level};
use zcash_keys::encoding::AddressCodec as _;
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

#[derive(Clone, Debug, Default)]
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

pub struct TxPlan {
    pub height: u32,
    pub inputs: Vec<TxPlanIn>,
    pub outputs: Vec<TxPlanOut>,
    pub fee: u64,
    pub can_sign: bool,
    pub can_broadcast: bool,
}

impl TxPlan {
    pub fn from_package(network: &Network, package: &PcztPackage) -> Result<Self> {
        let mut inputs = vec![];
        let mut outputs = vec![];

        let pczt = Pczt::parse(&package.pczt).unwrap();
        let height = *pczt.global().expiry_height();
        let verifier = Verifier::new(pczt);
        let mut fee = 0i64;

        let verifier = verifier
            .with_transparent(|bundle| {
                for i in bundle.inputs().iter() {
                    let value = i.value().into_u64();
                    inputs.push(TxPlanIn {
                        pool: 0,
                        amount: Some(value),
                    });
                    fee += value as i64;
                }
                for o in bundle.outputs().iter() {
                    let script_pubkey = o.script_pubkey();
                    outputs.push(TxPlanOut {
                        pool: 0,
                        amount: o.value().into_u64(),
                        address: script_pubkey.address().unwrap().encode(network),
                    });
                    fee -= o.value().into_u64() as i64;
                }
                Ok::<_, pczt::roles::verifier::TransparentError<()>>(())
            })
            .unwrap();

        let verifier = verifier
            .with_sapling(|bundle| {
                for _ in bundle.spends().iter() {
                    inputs.push(TxPlanIn {
                        pool: 1,
                        amount: None,
                    });
                }
                for o in bundle.outputs().iter() {
                    outputs.push(TxPlanOut {
                        pool: 1,
                        amount: o.value().unwrap().inner(),
                        address: o.user_address().as_ref().cloned().unwrap_or_default(),
                    });
                }
                fee += bundle.value_sum().to_raw() as i64;
                Ok::<_, pczt::roles::verifier::SaplingError<()>>(())
            })
            .unwrap();

        let _verifier = verifier
            .with_orchard(|bundle| {
                for a in bundle.actions().iter() {
                    inputs.push(TxPlanIn {
                        pool: 2,
                        amount: None,
                    });
                    outputs.push(TxPlanOut {
                        pool: 2,
                        amount: a.output().value().expect("value").inner(),
                        address: a
                            .output()
                            .user_address()
                            .as_ref()
                            .cloned()
                            .unwrap_or_default(),
                    });
                }
                let f: i64 = (*bundle.value_sum()).try_into().unwrap();
                fee += f;
                Ok::<_, pczt::roles::verifier::OrchardError<()>>(())
            })
            .unwrap();

        Ok(TxPlan {
            height,
            inputs,
            outputs,
            fee: fee as u64,
            can_sign: package.can_sign,
            can_broadcast: package.can_broadcast,
        })
    }
}

pub struct TxPlanIn {
    pub pool: u8,
    pub amount: Option<u64>,
}

pub struct TxPlanOut {
    pub pool: u8,
    pub amount: u64,
    pub address: String,
}

pub async fn send(client: &mut Client, height: u32, data: &[u8]) -> Result<String> {
    let span = span!(Level::INFO, "transaction");
    let txid = client.post_transaction(height, data).await?;
    span.in_scope(|| {
        info!("TXID: {}", txid);
    });
    Ok(txid)
}
