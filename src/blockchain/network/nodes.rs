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
    pub fn new(role: Role, profile: Profile) -> Result<Self> {
        if matches!(role, Role::Miner) {
            Self::genesis(profile)
        } else {
            Ok(Self {
                account: Account::new(role, profile),
                blockchain: RwLock::new(BlockChain::new()),
                loose_events: RwLock::new(Vec::new()),
            })
        }
    }

    /// Function to create a blockchain with a genesis block
    fn genesis(profile: Profile) -> Result<Self> {
        let account = Account::new(Role::Miner, profile);
        let mut initial_bc: BlockChain<T> = BlockChain::new();
        let genesis: Block<T> = Block::new(None);
        initial_bc.append(genesis, &account.priv_key, account.id)?;
        Ok(Self {
            account,
            blockchain: RwLock::new(initial_bc),
            loose_events: RwLock::new(Vec::new()),
        })
    }

    /// Gets the length of the underlying blockchain
    pub async fn bc_len(&self) -> usize {
        let unlocked = self.blockchain.read().await;
        unlocked.len()
    }

    /// Returns the overlap percentage between 2 [`BlockChain`] instances
    pub async fn chain_overlap(&self, chain: &BlockChain<T>) -> f64 {
        let unlocked = self.blockchain.read().await;
        unlocked.chain_overlap(chain)
    }

    /// Determines if a given block is in the underlying [`BlockChain']
    pub async fn in_chain(&self, block: &Block<T>) -> bool {
        let unlocked = self.blockchain.read().await;
        unlocked.in_chain(block)
    }

    /// Adds a [`Block`] to the underlying [`BlockChain`]
    pub async fn add_block(&self, block: Block<T>) -> Result<()> {
        let mut unlocked = self.blockchain.write().await;
        unlocked.append(block, &self.account.priv_key, self.account.id)
    }
}
