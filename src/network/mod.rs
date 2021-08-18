//! Main running code for all network interactions

mod accounts;
mod connections;
mod jobsync;
mod lookup;
pub mod messages;
mod nodes;
pub mod participants;

#[cfg(test)]
mod tests;

pub use self::{
    accounts::{Account, Role},
    connections::{Connection, ConnectionPool, Halves},
    jobsync::JobSync,
    lookup::lookup_run,
    messages::traits::{ReadLengthPrefix, WriteLengthPrefix},
    nodes::Node,
};

use crate::BlockChainBase;

use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tracing::debug;

/// Sends a new message to a [`Connection`] in [`ConnectionPool`]
///
/// Takes in a new [`NetworkMessage`] and distributes it to a [`Connection`] in the
/// [`ConnectionPool`] so they are aware of the information which is bein spread.
pub(crate) async fn send_message<T: BlockChainBase, S: AsyncWriteExt + Send + Unpin>(
    stream: &mut S,
    message: messages::NetworkMessage<T>,
) -> Result<()> {
    debug!("Sending Message: {:?}", &message);
    let bytes_message = message.as_bytes();
    stream.write_all(&bytes_message).await?;
    Ok(())
}
