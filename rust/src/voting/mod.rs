//! Zcash shielded voting (ZIP 262).
//!
//! Voting state lives in the crate's own rusqlite database beside the wallet
//! file rather than in zkool's SQLCipher pool, so `zcash_voting`'s sidecar
//! design is used throughout.

pub mod chain;
pub mod drive;
pub mod hotkey;
pub mod net;
pub mod sidecar;
pub mod summary;

pub mod note_source;
pub use note_source::ZkoolNoteSource;
