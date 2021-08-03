///! Module for messages sent in the network
mod network_message;
mod process_message;

pub use self::{
    network_message::{MessageData, NetworkMessage},
    process_message::ProcessMessage,
};
