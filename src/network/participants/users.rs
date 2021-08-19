//! Functionality specific to nodes with the [`Role::User`] role

use crate::{
    network::{
        messages::{MessageData, NetworkMessage, ProcessMessage},
        participants::shared::{add_block, replace_blockchain},
    },
    BlockChainBase, UserPair,
};

use std::sync::Arc;

use anyhow::{Error, Result};
use tracing::debug;

/// Deals with incoming messages from each [`Connection`] in the [`ConnectionPool`]
///
/// Listens to each [`Connection`] and consumes any messages from the associated [`TcpStream`].
/// This message is then dealt with. Each [`NetworkMessage`] is processed using a state machine
/// structure, which is best suited to the unpredictable nature of the incoming messages.
pub async fn users_state_machine<T: BlockChainBase + 'static>(
    pair: Arc<UserPair<T>>,
    message: NetworkMessage<T>,
) -> Result<()> {
    match &message.data {
        MessageData::Event(e) => {
            // If miner, add it to list of events to build Block
            // Else, pass onto other connections
            let mut ns_unlocked = pair.sync.nonce_set.write().await;
            if !ns_unlocked.contains(&e.nonce) {
                ns_unlocked.insert(e.nonce);
                match pair
                    .sync
                    .outbound_channel
                    .0
                    .send(ProcessMessage::SendMessage(message))
                {
                    Ok(_) => pair.sync.new_permit(),
                    Err(e) => return Err(Error::msg(format!("Error writing block: {}", e))),
                };
            }
            Ok(())
        }
        MessageData::Block(b) => {
            // Check if alreday in Blockchain
            // If not in blockchain (and is valid),
            if !pair.node.in_chain(&b).await {
                // add to blockchain
                add_block(Arc::clone(&pair), b).await?;
            }
            // Ignore if already in blockchain

            Ok(())
        }
        MessageData::State(bc) => {
            debug!("New blockchain received");
            // Check if valid
            if bc.validate_chain().is_ok() {
                debug!("New blockchain is valid");

                // If valid, check if it is a subchain of current blockchain
                if bc.len() > pair.node.bc_len().await && pair.node.chain_overlap(&bc).await > 0.5 {
                    // If longer and contains more than half of original chain, replace
                    replace_blockchain(Arc::clone(&pair), bc).await?;
                }
                // If shorter, ignore
            }
            // If not valid, ignore
            Ok(())
        }
        _ => Ok(()),
    }
}
