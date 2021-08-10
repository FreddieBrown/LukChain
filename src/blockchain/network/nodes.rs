//! Node structure containing all information for application

use crate::blockchain::{
    config::Profile,
    network::accounts::{Account, Role},
    Block, BlockChain, BlockChainBase, Event,
};

use std::fmt::Debug;

use anyhow::Result;
use tokio::sync::RwLock;

/// Struct to contain all information about `Node`
#[derive(Debug)]
pub struct Node<T: BlockChainBase> {
    pub account: Account,
    pub blockchain: RwLock<BlockChain<T>>,
    pub loose_events: RwLock<Vec<Event<T>>>,
}

impl<T: BlockChainBase> Node<T> {
    pub fn new(role: Role, profile: Profile) -> Self {
        Self {
            account: Account::new(role, profile),
            blockchain: RwLock::new(BlockChain::new()),
            loose_events: RwLock::new(Vec::new()),
        }
    }

    pub async fn bc_len(&self) -> usize {
        let unlocked = self.blockchain.read().await;
        unlocked.len()
    }

    pub async fn chain_overlap(&self, chain: &BlockChain<T>) -> f64 {
        let unlocked = self.blockchain.read().await;
        unlocked.chain_overlap(chain)
    }

    pub async fn in_chain(&self, block: &Block<T>) -> bool {
        let unlocked = self.blockchain.read().await;
        unlocked.in_chain(block)
    }

    pub async fn add_block(&self, block: Block<T>) -> Result<()> {
        let mut unlocked = self.blockchain.write().await;
        unlocked.append(block, &self.account.priv_key, self.account.id)
    }
}
