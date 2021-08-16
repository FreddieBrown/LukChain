//! Data structures and functions needed for the blockchain

mod blockchain;
pub mod config;
mod events;
pub mod network;
mod traits;
mod userpair;

#[cfg(test)]
mod tests;

pub use self::{
    blockchain::{Block, BlockChain},
    config::Profile,
    events::{Data, Event},
    network::{JobSync, Node, Role},
    traits::BlockChainBase,
    userpair::UserPair,
};
