//! Zcash shielded voting (ZIP 262).
//!
//! The redesign on zcash_voting 5.1.0 lands piece by piece, and only the
//! sidecar database and wallet note source are in the build. 5.1.0 dropped the
//! embedded-pool API (`VotingDb::from_pool`) the previous implementation was
//! built on: voting state now lives in the crate's own rusqlite database
//! beside the wallet file rather than in zkool's SQLCipher pool.
//!
//! The previous implementation is kept verbatim in `legacy.rs` and is not
//! compiled. Independent configuration transport and list-page summaries have
//! been ported to the sidecar design and remain available.

pub mod chain;
pub mod drive;
pub mod hotkey;
pub mod net;
pub mod sidecar;
pub mod summary;

pub mod note_source;
pub use note_source::ZkoolNoteSource;
