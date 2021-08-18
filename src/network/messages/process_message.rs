///! Messages sent between processes so they can trigger actions in other processes
use crate::network::messages::NetworkMessage;

use std::fmt::Debug;

use serde::{Deserialize, Serialize};

/// Messages to pass between threads to trigger behaviours
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ProcessMessage<T> {
    /// Base type, useless. Used to convey no information
    Blank,
    /// Indicates that a new connection should be created with node
    NewConnection(u128, String),
    /// Requests incuded [`NetworkMessage`] is sent to connected nodes
    SendMessage(NetworkMessage<T>),
}
