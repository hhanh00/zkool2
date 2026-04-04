use crate::{
    warp::{AuthPath, FragmentAuthPath, Hasher, Witness, MERKLE_DEPTH},
    Hash32,
};
use anyhow::Result;

pub struct MerklePath<const D: usize> {
    pub value: Hash32,
    pub position: u32,
    pub path: [Hash32; D],
}

impl AuthPath {
    pub fn root<H: Hasher>(&self, position: u32, value: &[u8; 32], h: &H) -> Hash32 {
        let mut hash = *value;
        let mut p = position;
        for (i, ommer) in self.0.iter().enumerate() {
            hash = if p & 1 == 0 {
                h.combine(i as u8, &hash, ommer)
            } else {
                h.combine(i as u8, ommer, &hash)
            };
            p /= 2;
        }
        hash
    }
}

// height at which a and b binary representation start to diverge
fn divergence_height(a: u32, b: u32) -> usize {
    assert!(a <= b, "{a} must be <= {b}");
    // after the xor, the common prefix becomes 0...
    let xor = a ^ b;
    (u32::BITS - xor.leading_zeros()) as usize
}

impl Witness {
    pub fn rewind(self, edge_position: u32) -> Self {
        let Witness {
            value,
            position,
            mut ommers,
            anchor,
        } = self;
        // calculate the height of the current partial subtree
        let h = divergence_height(self.position, edge_position);
        // clear right ommers that were filled after the position
        let mut p = self.position;
        for i in 0..MERKLE_DEPTH as usize {
            if i + 1 >= h && p & 1 == 0 {
                ommers.0[i] = None;
            }
            p /= 2;
        }
        Witness {
            value,
            position,
            ommers,
            anchor,
        }
    }

    pub fn build_auth_path(
        &self,
        edge: &FragmentAuthPath,
        empty_roots: &AuthPath,
    ) -> Result<AuthPath> {
        // calculate the height of the current partial subtree
        let h = divergence_height(self.position, edge.1);
        let mut path = AuthPath::default();
        let mut p = self.position;

        for i in 0..MERKLE_DEPTH as usize {
            path.0[i] = if p & 1 == 1 {
                // Right node: sibling must be known
                self.ommers.0[i].ok_or_else(|| {
                    anyhow::anyhow!("ommer at level {i} must be Some for right node")
                })?
            } else if i + 1 < h {
                // Left node below subtree height: the right tree must be full and therefore
                // known
                self.ommers.0[i].ok_or_else(|| {
                    anyhow::anyhow!("ommer at level {i} must be Some for left node")
                })?
            } else if i + 1 == h {
                anyhow::ensure!(self.ommers.0[i].is_none());
                // Left node at subtree height: use the partial subtree
                edge.0 .0[i]
            } else {
                anyhow::ensure!(self.ommers.0[i].is_none());
                // Left node above subtree height, the right node must be empty
                empty_roots.0[i]
            };
            p >>= 1;
        }
        Ok(path)
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
                        edge.0[i]
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
