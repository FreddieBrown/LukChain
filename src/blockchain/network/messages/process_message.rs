///! Messages sent between processes so they can trigger actions in other processes
use crate::blockchain::network::messages::NetworkMessage;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ProcessMessage {
    Blank,
    NewConnection(String),
    SendMessage(NetworkMessage),
}
