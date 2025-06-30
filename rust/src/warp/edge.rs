use super::{AuthPath, Edge, Hash32, Hasher};

impl Edge {
    pub fn root<H: Hasher>(&self, h: &H) -> Hash32 {
        let mut empty = h.empty();
        let mut hash = h.empty();
        for (depth, n) in self.0.iter().enumerate() {
            match n {
                Some(n) => {
                    hash = h.combine(depth as u8, n, &hash);
                }
                None => {
                    hash = h.combine(depth as u8, &hash, &empty);
                }
            }
            empty = h.combine(depth as u8, &empty, &empty);
        }
        hash
    }

    pub fn to_auth_path<H: Hasher>(&self, h: &H) -> AuthPath {
        let mut empty = h.empty();
        let mut hash = h.empty();
        let mut path = AuthPath::default();
        for (depth, n) in self.0.iter().enumerate() {
            path.0[depth] = hash;
            match n {
                Some(n) => {
                    hash = h.combine(depth as u8, n, &hash);
                }
                None => {
                    hash = h.combine(depth as u8, &hash, &empty);
                }
            }
            empty = h.combine(depth as u8, &empty, &empty);
        }
        path
    }
}
