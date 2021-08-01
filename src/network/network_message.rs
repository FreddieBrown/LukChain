use crate::blockchain::Event;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkMessage {
    pub data: MessageData,
}

impl NetworkMessage {
    pub fn new(data: MessageData) -> Self {
        Self { data }
    }

    // Function to convert struct to byte ready version to send
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum MessageData {
    Event(Event),
}
