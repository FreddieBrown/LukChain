//! Functionality specific to nodes with the [`Role::Miner`] role

use crate::blockchain::{
    network::{
        connections::ConnectionPool,
        messages::{MessageData, NetworkMessage, ProcessMessage},
        participants::shared::{
            check_connections, clear_connection_pool, incoming_connections, initial_lookup,
            outgoing_connections, setup,
        },
    },
    Block, BlockChainBase, Event, UserPair,
};

use std::sync::{atomic::Ordering, Arc};

use anyhow::{Error, Result};
use futures::join;
use tokio::net::TcpListener;
use tokio::task;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info};

/// Main runner function for participant functionality
///
/// Needs to have a defined base datatype for the [`BlockChain`] to
/// store. The default one in this librar is [`Data`]. Also provided
/// is the ability to pass in a function which can interact with the
/// [`JobSync`] mechanism. This lets the user build application
/// logic which can be used to do specific tasks, like create and
/// send messages to the outgoing thread.
pub async fn miners_run<T: 'static + BlockChainBase>(
    pair: Arc<UserPair<T>>,
    port: Option<u16>,
) -> Result<()> {
    let connect_pool: Arc<ConnectionPool> = Arc::new(ConnectionPool::new());

    let listener = setup(port).await?;

    let inbound_addr = listener.local_addr()?;

    info!("Participant Address: {:?}", &inbound_addr);

    // Talk to LookUp Server and get initial connections
    if let Some(addr) = pair.node.account.profile.lookup_address.clone() {
        match initial_lookup(Arc::clone(&pair), addr, inbound_addr.to_string()).await {
            Ok(_) => debug!("Initial Lookup Success"),
            Err(e) => error!("Initial Lookup Error: {}", e),
        };
    }

    let inbound_fut = inbound(Arc::clone(&pair), Arc::clone(&connect_pool), listener);

    let pair_cpy = Arc::clone(&pair);
    let outgoing_fut = tokio::spawn(async move {
        outgoing_connections(pair_cpy, Arc::clone(&connect_pool))
            .await
            .unwrap()
    });

    match join!(inbound_fut, outgoing_fut) {
        (Ok(_), Err(e)) => error!("Error in futures: {}", e),
        (Err(e), Ok(_)) => error!("Error in futures: {}", e),
        (Err(e1), Err(e2)) => error!("Multiple Errors in futures: {} and {}", e1, e2),
        _ => debug!("All fine!"),
    };

    Ok(())
}

/// Function to start up functions which deal with inbound
/// connections and message traffic
async fn inbound<T: 'static + BlockChainBase + Send>(
    pair: Arc<UserPair<T>>,
    connect_pool: Arc<ConnectionPool>,
    listener: TcpListener,
) -> Result<()> {
    // Thread to listen for inbound connections (reactive)
    //     Put all connections into connect pool
    let pair_cpy = Arc::clone(&pair);
    let cp_cpy = Arc::clone(&connect_pool);
    task::spawn(async move {
        incoming_connections(pair_cpy, cp_cpy, listener)
            .await
            .unwrap()
    });
    // Thread to go through all connections and deal with incoming messages (reactive)
    let _conns: Result<()> = task::spawn(async move {
        loop {
            let pair_clone = Arc::clone(&pair);
            let cp_clone = Arc::clone(&connect_pool);
            let cp_clear_status = pair_clone.sync.cp_clear.load(Ordering::SeqCst);
            let cp_size = pair_clone.sync.cp_size.load(Ordering::SeqCst);
            if cp_clear_status {
                // Clear connection pool of all dead connections
                clear_connection_pool(Arc::clone(&pair_clone), Arc::clone(&cp_clone)).await?;
                pair_clone.sync.cp_clear.store(false, Ordering::SeqCst);
            }

            if cp_size > 0 {
                check_connections(
                    Arc::clone(&pair_clone),
                    Arc::clone(&cp_clone),
                    miner_state_machine,
                )
                .await?
            } else {
                // Sleep for a moment (10 seconds?)
                debug!("NO CONNECTIONS");
                sleep(Duration::from_millis(1000)).await;
            }
        }
    })
    .await?;

    Ok(())
}

/// Deals with incoming messages from each [`Connection`] in the [`ConnectionPool`]
///
/// Listens to each [`Connection`] and consumes any messages from the associated [`TcpStream`].
/// This message is then dealt with. Each [`NetworkMessage`] is processed using a state machine
/// structure, which is best suited to the unpredictable nature of the incoming messages.
async fn miner_state_machine<T: BlockChainBase>(
    pair: Arc<UserPair<T>>,
    message: NetworkMessage<T>,
) -> Result<()> {
    match &message.data {
        MessageData::Event(e) => {
            // If miner, add it to list of events to build Block
            debug!("Recv Event: {:?}", e);
            let mut unlocked_events = pair.node.loose_events.write().await;
            let mut bc_unlocked = pair.node.blockchain.write().await;
            let mut ns_unlocked = pair.sync.nonce_set.write().await;
            // Check if event is already in loose_events and not in blockchain
            if !ns_unlocked.contains(&e.nonce)
                && !unlocked_events.contains(&e)
                && !bc_unlocked.contains(&e)
            {
                debug!("Event is new");
                // If it is not, add to set
                unlocked_events.push(e.clone());
                // if Vec over threshold size, build block and empty loose_events
                // TODO: Abstract out threshold size
                let thresh: usize = match pair.node.account.profile.block_size {
                    Some(s) => s,
                    None => 100,
                };

                if unlocked_events.len() >= thresh {
                    debug!("Building new block");
                    let last_hash = bc_unlocked.last_hash();

                    // Create new block and add to chain
                    let mut block: Block<T> = Block::new(last_hash);
                    block.add_events(unlocked_events.clone());
                    bc_unlocked.append(block.clone(), Arc::clone(&pair)).await?;

                    // Write updated chain to file
                    bc_unlocked.save()?;

                    // Reset events vector
                    *unlocked_events = Vec::new();
                    ns_unlocked.insert(e.nonce);
                    ns_unlocked.insert(block.nonce);
                    match pair
                        .sync
                        .outbound_channel
                        .0
                        .send(ProcessMessage::SendMessage(NetworkMessage::new(
                            MessageData::Block(block),
                        ))) {
                        Ok(_) => pair.sync.new_permit(),
                        Err(e) => return Err(Error::msg(format!("Error writing block: {}", e))),
                    };
                }
            }
            Ok(())
        }
        MessageData::Block(b) => {
            // Check if alreday in Blockchain
            // If not in blockchain (and is valid),
            if !pair.node.in_chain(&b).await {
                // add to blockchain
                pair.node.add_block(b.clone(), Arc::clone(&pair)).await?;
                // TODO: Go through loose events and remove any which are in block
                let mut unlocked_loose = pair.node.loose_events.write().await;
                let dropped = unlocked_loose
                    .drain_filter(|x| b.events.contains(x))
                    .collect::<Vec<Event<T>>>();

                debug!("Events found to be in block: {:?}", dropped);

                // pass onto other connected nodes
                let message: NetworkMessage<T> = NetworkMessage::new(MessageData::Block(b.clone()));
                let process_message: ProcessMessage<T> = ProcessMessage::SendMessage(message);
                match pair.sync.outbound_channel.0.send(process_message) {
                    Ok(_) => pair.sync.new_permit(),
                    Err(e) => return Err(Error::msg(format!("Error writing block: {}", e))),
                };
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
                    info!("New blockchain received, old Blockchain replaced");
                    let mut bc_unlocked = pair.node.blockchain.write().await;

                    // Set new save blockchain location to one of previous blockchain
                    let mut new_bc = bc.clone();
                    new_bc.set_save_location(bc_unlocked.save_location());

                    // Save new blockchain
                    *bc_unlocked = new_bc;
                    bc_unlocked.save()?;
                }
                // If shorter, ignore
            }
            // If not valid, ignore
            Ok(())
        }
        _ => Ok(()),
    }
}
