//! Main running code for all network interactions

mod accounts;
mod network_message;
mod nodes;

#[cfg(test)]
mod tests;

pub use self::{
    accounts::{Account, Role},
    network_message::{MessageData, NetworkMessage},
    nodes::Node,
};

use anyhow::Result;
use std::sync::Arc;

pub fn run(role: Role) -> Result<()> {
    let _node: Arc<Node> = Arc::new(Node::new(role));
    receiving().unwrap();
    sending().unwrap();
    Ok(())
}

fn receiving() -> Result<()> {
    Ok(())
}

fn sending() -> Result<()> {
    Ok(())
}

// Thread to listen for inbound connections (reactive)
//     Put all connections into connect pool
// Thread to forge outgoing connections and send outgoing messages (proactive)
//     Put all connections into connect pool
// Thread to go through all connections and deal with incoming messages (reactive)
