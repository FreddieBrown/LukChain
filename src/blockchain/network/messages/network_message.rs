///! Functionality for creating sendable messages across network.
use crate::blockchain::{network::Role, Block, BlockChain, BlockChainBase, Event};

use std::fmt::Debug;

use rsa::RsaPublicKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkMessage<T> {
    pub data: MessageData<T>,
}

impl<T: BlockChainBase> NetworkMessage<T> {
    pub fn new(data: MessageData<T>) -> Self {
        Self { data }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum MessageData<T> {
    // Blockchain specific Data
    Event(Event<T>),
    Block(Block<T>),
    State(BlockChain<T>),
    // Connection Configuration
    InitialID(u128, RsaPublicKey, Role),
    Confirm,
    Finish,
    // Lookup Table Requests
    LookUpReg(u128, String, Role),
    RequestAddress(u128),
    GeneralAddrRequest(Option<Role>),
    PeerAddresses(Vec<String>),
    PeerAddress(String),
    NoAddr,
    // Default Response
    Blank,
}
