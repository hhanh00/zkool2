use crate::Hash32;
#[cfg(feature = "imt")]
use orchard::tree::MerkleHashOrchard;
use std::io::{Read, Write};
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

    pub fn write<W: Write>(&self, mut writer: W) -> std::io::Result<()> {
        Optional::write(&mut writer, self.left.as_ref(), |w, h| w.write_all(h))?;
        Optional::write(&mut writer, self.right.as_ref(), |w, h| w.write_all(h))?;
        Vector::write(&mut writer, &self.parents, |w, p| {
            Optional::write(w, p.as_ref(), |w, h| w.write_all(h))
        })?;
        Ok(())
    }

    pub fn size(&self) -> usize {
        self.parents.iter().enumerate().fold(
            match (self.left.as_ref(), self.right.as_ref()) {
                (None, None) => 0,
                (Some(_), None) => 1,
                (Some(_), Some(_)) => 2,
                (None, Some(_)) => unreachable!(),
            },
            |acc, (i, p)| acc + if p.is_some() { 1 << (i + 1) } else { 0 },
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
        for (i, e) in edge.iter_mut().enumerate() {
            match (prev, carry) {
                (_, None) => {
                    *e = prev;
                }
                (None, Some(_)) => {
                    *e = carry;
                    carry = None;
                }
                (Some(l), Some(r)) => {
                    *e = None;
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

#[cfg(feature = "imt")]
pub type OrchardFrontier =
    incrementalmerkletree::frontier::Frontier<MerkleHashOrchard, MERKLE_DEPTH>;

#[cfg(feature = "imt")]
impl CommitmentTreeFrontier {
    pub fn from_orchard_frontier(frontier: &OrchardFrontier) -> Self {
        let Some(nef) = frontier.value() else {
            return CommitmentTreeFrontier::default();
        };

        let pos = u64::from(nef.position());
        let mut ommer_iter = nef.ommers().iter();
        let leaf = nef.leaf();

        let (left, right) = if pos % 2 == 0 {
            (Some(leaf.to_bytes()), None)
        } else {
            let left_bytes = ommer_iter.next().unwrap().to_bytes();
            (Some(left_bytes), Some(leaf.to_bytes()))
        };

        let mut parents: Vec<Option<Hash32>> = vec![];
        for i in 0..(MERKLE_DEPTH as usize - 1) {
            if (pos >> (i + 1)) & 1 == 1 {
                parents.push(Some(ommer_iter.next().unwrap().to_bytes()));
            } else {
                parents.push(None);
            }
        }

        CommitmentTreeFrontier { left, right, parents }
    }

    pub fn to_orchard_frontier(self) -> OrchardFrontier {
        let size = self.size();
        if size == 0 {
            return OrchardFrontier::empty();
        }
        let CommitmentTreeFrontier {
            left,
            right,
            parents,
        } = self;

        let position = incrementalmerkletree::Position::from((size - 1) as u64);
        let mut ommers = vec![];
        let leaf = match (left, right) {
            (Some(l), None) => MerkleHashOrchard::from_bytes(&l).unwrap(),
            (Some(l), Some(r)) => {
                ommers.push(MerkleHashOrchard::from_bytes(&l).unwrap());
                MerkleHashOrchard::from_bytes(&r).unwrap()
            }
            _ => unreachable!("non-empty frontier must have at least left"),
        };
        for p in parents.into_iter().flatten() {
            ommers.push(MerkleHashOrchard::from_bytes(&p).unwrap());
        }

        OrchardFrontier::from_parts(position, leaf, ommers).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use bincode::config::legacy;

    use crate::warp::{
        hasher::{empty_roots, OrchardHasher},
        legacy::CommitmentTreeFrontier, Witness,
    };

    #[cfg(feature = "imt")]
    #[test]
    fn tree_state() {
        use crate::warp::hasher::OrchardHasher;

        let hasher = OrchardHasher::default();
        // Tree state at block 3007417 (fetched from lightwalletd)
        let orchard_tree_state = hex::decode("01c0cc5ce20c3764f87e1d9b2ca6dd3da9d3beb550f1602ccf7819f587761d823e001f000100e764f4106083cc27d6510c3fd316465fd17dc18d62d417b8739e5b3eece83301ac98b7ce63234dad2012adc6d836a7ae861018355b8bcb9a56aab3a74e390c0a0000011a08f67ce86a92effec4458bed582ce3502e5c33ad70dfd78edc2b6053bd952b01d9539b0c3c451b3601aeec7e8092f8a2faf7bd750aa245c0f48d0b6c3891a91301e8896c324fcef1e2534b832dc9c2b97746e1cc45e3e48ff1a2411a22be5692020177e98d01d86f79f59f79852100a9b7395ffc1d2598a5565fb65ebae7ccd2b638000000017a7bd62cf54ed2ed18d1b0e56f209a169a9b047b425c7e5038c22bdc82dcf0170163275c8cf2223bdb4aa14b1c86d140c5e30db04bee30abd21c94d0852485012601965f70049519600185d1a3ba0c418edc1da0bb2d978da1e3ceec93b8851af419014c093ed9408ece3d125981e3d091e9c811ba0472a15ed8a14eca311642d7ec3100012d113bc8f6a4f41b3963cfa0717176c2d31ce7bfae4d250a1fff5e061dd9d3250160040850b766b126a2b4843fcdfdffa5d5cab3f53bc860a3bef68958b5f066170001cc2dcaa338b312112db04b435a706d63244dd435238f0aa1e9e1598d35470810012dcc4273c8a0ed2337ecf7879380a07e7d427c7f9d82e538002bd1442978402c01daf63debf5b40df902dae98dadc029f281474d190cddecef1b10653248a234150001e2bca6a8d987d668defba89dc082196a922634ed88e065c669e526bb8815ee1b000000000000").unwrap();
        let orchard_tree_state =
            CommitmentTreeFrontier::read(orchard_tree_state.as_slice()).unwrap();
        let edge = orchard_tree_state.to_edge(&hasher);
        let root0 = edge.root(&hasher);

        let imt = orchard_tree_state.to_orchard_frontier();
        let root1 = imt.root().to_bytes();

        assert_eq!(root0, root1);
        println!("root: {}", hex::encode(root0));
    }

    #[cfg(feature = "imt")]
    #[test]
    fn rewind() {
        // actual witness data from real note

        // witness at height 3291281 for a note
        // this must be a height greater than the one we rewind to

        let witness = hex::decode("343664116CE838700BBB803556136D4C11B7FF4CAC7CCB3893F8DC20C195470BB483F502015B607DD53B4C8BEBF730047E6B6AAD41E6755112725FC8EFDD4B95FA02D6D3070185A43C44C2CE1F7B95A845A8CCD1B77375AA521BA0F119DA83797A4A5F618B3401E3774C31A10A3622E92FE380FC3C05CDEE0B2B5A4827C7C20E50999E80997E39012970A4E98F6B0DB36380437D391BB9F2A4BDF3AD22C477916AA9A23841E01C3601AC86FE3103502AD0DEE481E0F8B2D5AFEBEEE5E3BA51C3654779179A1FCBDF310153C81D479AD2111644616814F3B1C0614E53EC3435E9873DFB0722421DD5E9020164B671EF44F75A37BC6677AAD2F72062FFFC10B0933941087DD61951FDA4B32401BF05A2E6DB57081595138D60D7C97A1311A67EDDD7919B24FE68E633846B011301AA750059EB14FDCB785A89F3E0B04AE7DD4129A45CFF6DFA748764BDA00FB20D01674A12D6152EC288DD7552F0967492B921965E64B48EE58DD705500D6DB5BE3401A9ABD4B7E6038D5B050090BD039E9EED6ED80AD4B885BF5583B818595B3A8E0E01E4E4C3CBAA0CE3969E51C52E22357F9B7B5131069AACB6BB1D4766D82A86E5390162E4FAB9738EC103E3CAD058F70535BFE04C0BE3912F4327064D3BD0EB7607170168C70212CA93DE252D301E016A8F8AFFDDBE0B94BCDF67E6B8D95CDCDCF2E12601C0D221B2DF44394C6867388C86D996C8C7DF1F89A130E62EE7D9B73721478911015406770A790A86A2E97D198EAEA800387E94FCCF29916527BE06EE5AA54BB6120119AEFA4C6168DE8975F7A49948CB73856D6AD7E25ECA3AB2F8F371A922EE7D2B015FA73F204EB23E5D3C7278F4480300FA07ACABD14FFF9E1E337151D7F594E80701EAC2B89B3F966D833626434DF98D553E000324BBAFB8D6E1FE03B8D7F854CF2A00017C8ECE2B2AB2355D809B58809B21C7A5E95CFC693CD689387F7533EC8749261E01CC2DCAA338B312112DB04B435A706D63244DD435238F0AA1E9E1598D35470810012DCC4273C8A0ED2337ECF7879380A07E7D427C7F9D82E538002BD1442978402C01DAF63DEBF5B40DF902DAE98DADC029F281474D190CDDECEF1B10653248A234150001E2BCA6A8D987D668DEFBA89DC082196A922634ED88E065C669E526BB8815EE1B00000000000069771157B17B027800568FEEB7E2DF1140C3471CDE2D98DC856C8216C228161A").unwrap();
        let (witness, _) = bincode::decode_from_slice::<Witness, _>(&witness, legacy()).unwrap();
        let hasher = OrchardHasher::default();
        let empty_roots = empty_roots(&hasher);

        // tree state at 3240000
        let orchard_tree_state = hex::decode("01c324325bc50e80055a09c2fa1defaf4e35e7a2a4a0bed98aa01ad5a15c51ab3d001f0001ec6a1938e932af981679018acba6febe7fdcf3db817e8d0c680d1b70137bc73f011a42c5862f68c42e3ac35ee7888d5729ec24814d62f9802c856c1842ec98450d000001b8b6e33fbb3a2035e99ca74f23bb0ff777d128b3fa2d7d5a02a1e2902c77a220000001b34d56d339b39e8d33d24900d221bd9fd742b50c18e83f4e477ade4805f65013000189f756fdf90d0faaf9a7933a180be047a23e5a93f7c0b8d5477b506ee1e9a6330000017e172d7cfd8636a30fbc12579ed7e389b310c896e0013965fb1feeb30dd7542601e780c8897bdcc43041b643810bd50f3502b7a199dc763c2270daf494cfde2a3c0001fb82740a3629216088191f9cd359c52a2f35b1c58f6cc905781bd9687b66ad3801eac2b89b3f966d833626434df98d553e000324bbafb8d6e1fe03b8d7f854cf2a00017c8ece2b2ab2355d809b58809b21c7a5e95cfc693cd689387f7533ec8749261e01cc2dcaa338b312112db04b435a706d63244dd435238f0aa1e9e1598d35470810012dcc4273c8a0ed2337ecf7879380a07e7d427c7f9d82e538002bd1442978402c01daf63debf5b40df902dae98dadc029f281474d190cddecef1b10653248a234150001e2bca6a8d987d668defba89dc082196a922634ed88e065c669e526bb8815ee1b000000000000").unwrap();
        let orchard_tree_state =
            CommitmentTreeFrontier::read(orchard_tree_state.as_slice()).unwrap();
        let edge = orchard_tree_state.to_edge(&hasher);
        let edge_auth_path = edge.to_auth_path(&hasher);
        let root0 = edge.root(&hasher);
        // root at the rewound point
        println!("root0: {}", hex::encode(root0));

        let witness = witness.rewind(edge_auth_path.1);
        let path = witness.build_auth_path(&edge_auth_path, &empty_roots).unwrap();
        let root1 = path.root(witness.position, &witness.value, &hasher);

        // new witness should have the same root as the rewound point
        println!("root1: {}", hex::encode(root1));
        assert_eq!(root0, root1);
    }
}
