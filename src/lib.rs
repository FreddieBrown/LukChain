//! Generalised BlockChain with associated network functionality and example chat application
#![feature(drain_filter)]
#![warn(rust_2018_idioms)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]

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
