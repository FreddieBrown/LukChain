///! Messages sent between processes so they can trigger actions in other processes
use crate::blockchain::network::messages::NetworkMessage;

use std::fmt::Debug;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ProcessMessage<T> {
    Blank,
    NewConnection(String),
    SendMessage(NetworkMessage<T>),
}
