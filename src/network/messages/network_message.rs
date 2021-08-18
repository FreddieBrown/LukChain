///! Functionality for creating sendable messages across network.
use crate::{network::Role, Block, BlockChain, BlockChainBase, Event};

use std::fmt::Debug;

use rsa::RsaPublicKey;
use serde::{Deserialize, Serialize};

/// Struct to serialise/deserialise data to be sent between nodes
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkMessage<T> {
    /// Data to be sent in message
    pub data: MessageData<T>,
}

impl<T: BlockChainBase> NetworkMessage<T> {
    /// Creates new message to send across network interface
    pub fn new(data: MessageData<T>) -> Self {
        Self { data }
    }
}

/// Wrapper struct around data to be sent across network
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum MessageData<T> {
    // Blockchain specific Data
    /// Wrapper for [`Event`] struct
    Event(Event<T>),
    /// Wrapper for [`Block`] struct
    Block(Block<T>),
    /// Wrapper for [`BlockChain`] struct
    State(BlockChain<T>),
    // Connection Configuration
    /// Used to send information to external node on connection
    InitialID(u128, RsaPublicKey, Role),
    /// Used to confirm message has been received during communication transactions
    Confirm,
    /// Used to indicate connection should end
    Finish,
    // Lookup Table Requests
    /// Registers node in LookUp table
    LookUpReg(u128, String, Role),
    /// Requests the address of a node with a specific ID
    RequestAddress(u128),
    /// Request for up to 4 random nodes, with an optional filter for their roles
    GeneralAddrRequest(u128, Option<Role>),
    /// Returned addresses of external nodes to connect to
    PeerAddresses(Vec<(u128, String)>),
    /// Address of single node that was requested
    PeerAddress((u128, String)),
    /// Sent to LookUp to indicate poor behaviour by node
    Strike(u128),
    /// Sent by LookUp node when there are no addresses which match criteria in request
    NoAddr,
    /// Default Response
    Blank,
}
