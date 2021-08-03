///! Messages sent between processes so they can trigger actions in other processes
use crate::network::messages::NetworkMessage;

pub enum ProcessMessage {
    Blank,
    NewConnection,
    SendMessage(NetworkMessage),
}
