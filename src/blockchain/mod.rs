//! Data structures and functions needed for the blockchain

mod blockchain;
pub mod config;
mod events;
pub mod network;
mod traits;

#[cfg(test)]
mod tests;

pub use self::{
    blockchain::{Block, BlockChain},
    events::{Data, Event},
    traits::BlockChainBase,
};

use self::config::Profile;
use self::network::{JobSync, Node, Role};

use anyhow::Result;

pub struct UserPair<T: BlockChainBase> {
    pub sync: JobSync<T>,
    pub node: Node<T>,
}

impl<T: BlockChainBase> UserPair<T> {
    pub async fn new(role: Role, profile: Profile, write_back: bool) -> Result<Self> {
        let sync = JobSync::new(write_back);
        let node: Node<T> = if matches!(role, Role::Miner) {
            Node::genesis(profile.clone(), &sync).await?
        } else {
            Node::new(role, profile.clone())
        };

        Ok(Self { sync, node })
    }
}
