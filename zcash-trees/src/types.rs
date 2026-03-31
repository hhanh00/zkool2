use crate::warp::Witness;

#[derive(thiserror::Error, Debug)]
pub enum SyncError {
    #[error("Reorganization detected at block {0}")]
    Reorg(u32),
    #[error("Sync cancelled")]
    Cancelled,
    #[error("Tonic error: {0}")]
    Tonic(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Default, Debug)]
pub struct Transaction {
    pub id: u32,
    pub txid: Vec<u8>,
    pub height: u32,
    pub account: u32,
    pub time: u32,
    pub value: i64,
}

impl std::fmt::Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "txid {} {} @{}",
            hex::encode(&self.txid),
            self.account,
            self.height
        )
    }
}

#[derive(Default)]
pub struct UTXO {
    pub id: u32,
    pub pool: u8,
    pub account: u32,
    pub nullifier: Vec<u8>,
    pub value: u64,
    pub position: u32,
    pub cmx: Vec<u8>,
    pub witness: Witness,
    pub txid: Vec<u8>,
}

impl std::fmt::Display for UTXO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.account,
            self.pool,
            hex::encode(&self.nullifier)
        )
    }
}

impl std::fmt::Debug for UTXO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UTXO")
            .field("id", &self.id)
            .field("account", &self.account)
            .field("pool", &self.pool)
            .field("txid", &hex::encode(&self.txid))
            .field("cmx", &hex::encode(&self.cmx))
            .finish()
    }
}

pub struct BlockHeader {
    pub height: u32,
    pub hash: Vec<u8>,
    pub time: u32,
}

impl std::fmt::Debug for BlockHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlockHeader")
            .field("height", &self.height)
            .field("hash", &hex::encode(&self.hash))
            .finish()
    }
}

#[derive(Clone, Default)]
pub struct Note {
    pub id: u32,
    pub account: u32,
    pub scope: u8,
    pub height: u32,
    pub position: u32,
    pub pool: u8,
    pub id_tx: u32,
    pub vout: u32,
    pub diversifier: Vec<u8>,
    pub value: u64,
    pub rcm: Vec<u8>,
    pub rho: Vec<u8>,
    pub nf: Vec<u8>,
    pub ivtx: u32,
    pub cmx: Vec<u8>,
    pub txid: Vec<u8>,
}

impl std::fmt::Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.account,
            self.height,
            self.position,
            self.pool,
            hex::encode(&self.nf)
        )
    }
}

impl std::fmt::Debug for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Note")
            .field("id", &self.id)
            .field("account", &self.account)
            .field("height", &self.height)
            .field("position", &self.position)
            .field("pool", &self.pool)
            .field("id_tx", &self.id_tx)
            .field("vout", &self.vout)
            .field("diversifier", &hex::encode(&self.diversifier))
            .field("value", &self.value)
            .field("rcm", &hex::encode(&self.rcm))
            .field("rho", &hex::encode(&self.rho))
            .field("nf", &hex::encode(&self.nf))
            .finish()
    }
}

#[derive(Debug)]
pub enum WarpSyncMessage {
    BlockHeader(BlockHeader),
    Transaction(Transaction),
    Note(Note),
    Witness(u32, u32, Vec<u8>, Witness),
    Checkpoint(Vec<u32>, u8, u32),
    Commit,
    Spend(UTXO),
    Rewind(Vec<u32>, u32),
    Error(SyncError),
}

impl std::fmt::Display for WarpSyncMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WarpSyncMessage::BlockHeader(bh) => {
                write!(f, "Header: {} {}", bh.height, hex::encode(&bh.hash))
            }
            WarpSyncMessage::Transaction(tx) => write!(f, "Tx: {tx}"),
            WarpSyncMessage::Note(note) => write!(f, "Note: {note}"),
            WarpSyncMessage::Witness(account, height, cmx, witness) => write!(
                f,
                "Witness for {account} @{height}: {} {witness}",
                hex::encode(cmx)
            ),
            WarpSyncMessage::Checkpoint(_, pool, height) => {
                write!(f, "Checkpoint for {pool} @{height}")
            }
            WarpSyncMessage::Commit => write!(f, "Commit"),
            WarpSyncMessage::Spend(utxo) => write!(f, "Spend {utxo}"),
            WarpSyncMessage::Rewind(_, height) => write!(f, "Rewind to @{height}"),
            WarpSyncMessage::Error(e) => write!(f, "SyncError: {e:?}"),
        }
    }
}
