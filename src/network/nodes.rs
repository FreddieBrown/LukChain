//! Node structure containing all information for application

use std::sync::RwLock;

use crate::blockchain::BlockChain;
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
}
