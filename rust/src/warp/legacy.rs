use crate::Hash32;
use std::io::Read;
use zcash_encoding::{Optional, Vector};

use super::{Edge, Hasher, MERKLE_DEPTH};

#[derive(Default, Debug)]
pub struct CommitmentTreeFrontier {
    pub left: Option<Hash32>,
    pub right: Option<Hash32>,
    pub parents: Vec<Option<Hash32>>,
}

impl CommitmentTreeFrontier {
    pub fn read<R: Read>(mut reader: R) -> std::io::Result<Self> {
        let left = Optional::read(&mut reader, Self::hash_read)?;
        let right = Optional::read(&mut reader, Self::hash_read)?;
        let parents = Vector::read(&mut reader, |r| Optional::read(r, Self::hash_read))?;

        Ok(CommitmentTreeFrontier {
            left,
            right,
            parents,
        })
    }

    pub fn size(&self) -> usize {
        self.parents.iter().enumerate().fold(
            match (self.left.as_ref(), self.right.as_ref()) {
                (None, None) => 0,
                (Some(_), None) => 1,
                (Some(_), Some(_)) => 2,
                (None, Some(_)) => unreachable!(),
            },
            |acc, (i, p)| {
                // Treat occupation of parents array as a binary number
                // (right-shifted by 1)
                acc + if p.is_some() { 1 << (i + 1) } else { 0 }
            },
        )
    }

    fn hash_read<R: Read>(mut reader: R) -> std::io::Result<Hash32> {
        let mut repr = [0u8; 32];
        reader.read_exact(&mut repr)?;
        Ok(repr)
    }

    pub fn to_edge<H: Hasher>(&self, h: &H) -> Edge {
        let mut edge = [None; MERKLE_DEPTH as usize];
        let mut prev = self.left;
        let mut carry = self.right;
        for i in 0..MERKLE_DEPTH as usize {
            match (prev, carry) {
                (_, None) => {
                    edge[i] = prev;
                }
                (None, Some(_)) => {
                    edge[i] = carry;
                    carry = None;
                }
                (Some(l), Some(r)) => {
                    edge[i] = None;
                    carry = Some(h.combine(i as u8, &l, &r));
                }
            }
            prev = if i < self.parents.len() {
                self.parents[i]
            } else {
                None
            };
        }
        Edge(edge)
    }
}

pub struct CommitmentWitness {}
