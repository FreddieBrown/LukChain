///! Functionality for creating sendable messages across network.
use crate::blockchain::{Block, BlockChain, Event};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkMessage {
    pub data: MessageData,
}

impl NetworkMessage {
    pub fn new(data: MessageData) -> Self {
        Self { data }
    }

    /// Reads a [`NetworkMessage`] from a async stream
    pub async fn from_stream<R: AsyncReadExt + Send + Unpin>(
        stream: &mut R,
        mut buffer: &mut [u8],
    ) -> Result<Self> {
        // Read the size of the message
        let mut size_buffer = [0_u8; 4];
        stream.read_exact(&mut size_buffer).await?;

        // Convert it to a u32
        let message_size = u32::from_be_bytes(size_buffer);

        // Read again from the stream, extending the vector if needed
        let mut bytes = Vec::new();
        let mut remaining_size = message_size;

        // Enforce only reading the given size
        let mut truncated = stream.take(u64::from(remaining_size));

        while remaining_size != 0 {
            let size = truncated.read(&mut buffer).await?;
            bytes.extend_from_slice(&buffer[..size]);
            remaining_size -= size as u32;
        }

        Ok(serde_json::from_slice(&bytes)?)
    }

    /// Converts a [`NetworkMessage`] into a sendable form in bytes
    pub fn as_bytes(&self) -> Vec<u8> {
        // Convert the message to bytes
        let bytes = serde_json::to_vec(&self).unwrap();

        // Prepend with the length
        let length = bytes.len() as u32;

        let mut message = length.to_be_bytes().to_vec();
        message.extend(bytes);

        message
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum MessageData {
    Event(Event),
    Block(Block),
    State(BlockChain),
    InitialID(u128),
    Blank,
}
