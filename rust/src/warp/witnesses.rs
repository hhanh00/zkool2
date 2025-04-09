// use anyhow::Result;
// use incrementalmerkletree::witness::IncrementalWitness;
// use rusqlite::Connection;
// use sapling_crypto::Node;
// use tracing::info;

use crate::{
    warp::{AuthPath, Hasher, Witness, MERKLE_DEPTH},
    Hash32,
};

impl Witness {
    pub fn build_auth_path(&self, edge: &AuthPath, empty_roots: &AuthPath) -> AuthPath {
        let mut path = AuthPath::default();
        let mut p = self.position;
        let mut edge_used = false;
        for i in 0..MERKLE_DEPTH as usize {
            let ommer = self.ommers.0[i];
            path.0[i] = match ommer {
                Some(o) => o,
                None => {
                    assert!(p & 1 == 0);
                    if edge_used {
                        empty_roots.0[i]
                    } else {
                        edge_used = true;
                        edge.0[i]
                    }
                }
            };
            p /= 2;
        }
        path
    }

    pub fn root<H: Hasher>(&self, edge: &AuthPath, h: &H) -> Hash32 {
        let mut hash = self.value;
        let mut p = self.position;
        let mut empty = h.empty();
        let mut edge_used = false;
        for i in 0..MERKLE_DEPTH as usize {
            let ommer = self.ommers.0[i];
            hash = match ommer {
                Some(o) => {
                    if p & 1 == 0 {
                        h.combine(i as u8, &hash, &o)
                    } else {
                        h.combine(i as u8, &o, &hash)
                    }
                }
                None => {
                    assert!(p & 1 == 0);
                    let o = if edge_used {
                        empty
                    } else {
                        edge_used = true;
                        edge.0[i as usize]
                    };
                    h.combine(i as u8, &hash, &o)
                }
            };
            empty = h.combine(i as u8, &empty, &empty);
            p /= 2;
        }
        hash
    }
}
