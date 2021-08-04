//! Node structure containing all information for application

use anyhow::Result;
use tokio::sync::RwLock;

use crate::blockchain::{Block, BlockChain};
use crate::network::accounts::{Account, Role};

/// Struct to contain all information about `Node`
#[derive(Debug)]
pub struct Node {
    pub account: Account,
    pub blockchain: RwLock<BlockChain>,
}

impl Node {
    pub fn new(role: Role) -> Self {
        Self {
            account: Account::new(role),
            blockchain: RwLock::new(BlockChain::new()),
        }
    }

    pub async fn bc_len(&self) -> usize {
        let unlocked = self.blockchain.read().await;
        unlocked.len()
    }

    pub async fn chain_overlap(&self, chain: &BlockChain) -> f64 {
        let unlocked = self.blockchain.read().await;
        unlocked.chain_overlap(chain)
    }

    pub async fn in_chain(&self, block: &Block) -> bool {
        let unlocked = self.blockchain.read().await;
        unlocked.in_chain(block)
    }

    pub async fn add_block(&self, block: Block) -> Result<()> {
        let mut unlocked = self.blockchain.write().await;
        unlocked.append(block)
    }
}
