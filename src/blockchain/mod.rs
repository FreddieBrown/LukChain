//! Data structures and functions needed for the blockchain

mod blockchain;
pub mod config;
mod events;
pub mod network;

#[cfg(test)]
mod tests;

pub use self::{
    blockchain::{Block, BlockChain},
    events::{Data, Event},
};
