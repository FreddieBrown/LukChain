//! Main running code for all network interactions

pub mod accounts;
pub mod nodes;
#[cfg(test)]
mod tests;

use crate::network::{accounts::Role, nodes::Node};

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
