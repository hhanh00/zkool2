// UNUSED

use std::collections::HashMap;

use anyhow::Result;
use rusqlite::Connection;

use crate::{db::tx::store_block_time, warp::BlockHeader};

pub struct BlockHeaderStore {
    pub heights: HashMap<u32, Option<BlockHeader>>,
}

impl BlockHeaderStore {
    pub fn new() -> Self {
        Self {
            heights: HashMap::new(),
        }
    }

    pub fn add_heights<'a>(&mut self, heights: impl IntoIterator<Item = &'a u32>) -> Result<()> {
        for h in heights {
            self.heights.insert(*h, None);
        }
        Ok(())
    }

    pub fn process(&mut self, header: &BlockHeader) -> Result<()> {
        if self.heights.contains_key(&header.height) {
            self.heights.insert(header.height, Some(header.clone()));
        }
        Ok(())
    }

    pub fn save(&self, connection: &Connection) -> Result<()> {
        for (height, header) in self.heights.iter() {
            if let Some(header) = header {
                let timestamp = header.timestamp;
                store_block_time(connection, *height, timestamp)?;
            }
        }
        Ok(())
    }
}
