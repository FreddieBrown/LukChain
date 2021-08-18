//! Functionality shared between [`Role::User`] and [`Role::Miner`]
use crate::{
    network::{
        accounts::Role,
        connections::{Connection, ConnectionPool, Halves},
        messages::{traits::ReadLengthPrefix, MessageData, NetworkMessage, ProcessMessage},
        send_message,
    },
    Block, BlockChain, BlockChainBase, Event, UserPair,
};

use std::future::Future;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::ops::DerefMut;
use std::sync::{atomic::Ordering, Arc};

use anyhow::{Error, Result};
use futures::join;
use rsa::RsaPublicKey;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info};

/// Sets up [`TcpListener`] for running functions
pub async fn setup(port: Option<u16>) -> Result<TcpListener, std::io::Error> {
    #[cfg(not(debug_assertions))]
    let ip = Ipv4Addr::UNSPECIFIED;

    #[cfg(debug_assertions)]
    let ip = Ipv4Addr::LOCALHOST;

    // Open socket and start listening
    let socket = match port {
        Some(p) => SocketAddr::V4(SocketAddrV4::new(ip, p)),
        _ => SocketAddr::V4(SocketAddrV4::new(ip, 0)),
    };

    TcpListener::bind(socket).await
}

/// Connects to the lookup server and creates messages to form connections
///
/// Node registers itself with the LookUp server and requests connections for
/// it to make. It then creates a [`ProcessMessage::NewConnection`] for each
/// address and puts them into the job queue.
pub async fn initial_lookup<T: 'static + BlockChainBase>(
    pair: Arc<UserPair<T>>,
    address: String,
    own_addr: String,
) -> Result<()> {
    let mut buffer = [0_u8; 4096];
    debug!("Creating LookUp Connection with: {}", address);
    // Open connection
    let mut stream: TcpStream = TcpStream::connect(address).await?;

    let reg_message = NetworkMessage::<T>::new(MessageData::LookUpReg(
        pair.node.account.id,
        own_addr,
        pair.node.account.role,
    ));

    send_message(&mut stream, reg_message).await?;

    debug!("Sent Reg Message");

    if matches!(
        NetworkMessage::<T>::from_stream(&mut stream, &mut buffer)
            .await?
            .data,
        MessageData::Confirm
    ) {
        // Create general request message and send over stream
        let gen_req = NetworkMessage::<T>::new(MessageData::GeneralAddrRequest(
            pair.node.account.id,
            pair.node.account.profile.lookup_filter,
        ));

        send_message(&mut stream, gen_req).await?;

        debug!("Sent Message");

        let read_in = NetworkMessage::<T>::from_stream(&mut stream, &mut buffer).await?;

        debug!("Read Message");

        match read_in.data {
            MessageData::PeerAddresses(addrs) => {
                // Iterate over addresses and create connection requests
                for (id, addr) in addrs {
                    debug!("Creating Connection Message for: {}", &addr);

                    let process_message = ProcessMessage::NewConnection(id, addr);

                    match pair.sync.outbound_channel.0.send(process_message) {
                        Ok(_) => pair.sync.new_permit(),
                        Err(e) => return Err(Error::msg(format!("Error writing block: {}", e))),
                    };
                }
            }
            MessageData::NoAddr => {
                debug!("No addresses in the lookup table");
            }
            _ => unreachable!(),
        }
    }

    let finish_message = NetworkMessage::<T>::new(MessageData::Finish);
    send_message(&mut stream, finish_message).await?;

    debug!("FINISHED INITIAL LOOKUP");

    Ok(())
}

/// Connects to the lookup server and creates messages to form connections
///
/// Node connects to specific [`LookUp`] node and sends a [`GeneralAddrRequest`].
/// The node will return the result of the query depending on what is contained within
/// its database.
async fn general_lookup<T: 'static + BlockChainBase>(
    pair: Arc<UserPair<T>>,
    address: String,
) -> Result<()> {
    let mut buffer = [0_u8; 4096];
    debug!("GENERAL LOOKUP");
    debug!("Creating LookUp Connection with: {}", address);
    // Open connection
    let mut stream: TcpStream = TcpStream::connect(address).await?;

    // Create general request message and send over stream
    let gen_req = NetworkMessage::<T>::new(MessageData::GeneralAddrRequest(
        pair.node.account.id,
        pair.node.account.profile.lookup_filter,
    ));

    send_message(&mut stream, gen_req).await?;

    debug!("Sent Message");

    let read_in = NetworkMessage::<T>::from_stream(&mut stream, &mut buffer).await?;

    debug!("Read Message");

    match read_in.data {
        MessageData::PeerAddresses(addrs) => {
            // Iterate over addresses and create connection requests
            for (id, addr) in addrs {
                debug!("Creating Connection Message for: {}", &addr);

                let process_message = ProcessMessage::NewConnection(id, addr);

                match pair.sync.outbound_channel.0.send(process_message) {
                    Ok(_) => pair.sync.new_permit(),
                    Err(e) => return Err(Error::msg(format!("Error writing block: {}", e))),
                };
            }
        }
        MessageData::NoAddr => {
            let finish_message = NetworkMessage::<T>::new(MessageData::Finish);
            send_message(&mut stream, finish_message).await?;
            return Err(Error::msg("No addresses in the lookup table"));
        }
        _ => unreachable!(),
    }

    let finish_message = NetworkMessage::<T>::new(MessageData::Finish);
    send_message(&mut stream, finish_message).await?;

    Ok(())
}

/// Connects to the lookup server and sends a [`Strike`] message
async fn lookup_strike<T: 'static + BlockChainBase>(address: String, conn_id: u128) -> Result<()> {
    debug!("SENDING STRIKE");
    debug!("Creating LookUp Connection with: {}", address);
    // Open connection
    let mut stream: TcpStream = TcpStream::connect(address).await?;

    // Create general request message and send over stream
    let gen_req = NetworkMessage::<T>::new(MessageData::Strike(conn_id));

    send_message(&mut stream, gen_req).await?;

    debug!("Sent Message");

    Ok(())
}

pub async fn clear_connection_pool<T: 'static + BlockChainBase + Send>(
    pair: Arc<UserPair<T>>,
    connect_pool: Arc<ConnectionPool>,
) -> Result<()> {
    let mut cp_buffer_write = pair.sync.cp_buffer.write().await;
    let size = cp_buffer_write.len();
    let mut map_write = connect_pool.map.write().await;

    debug!("CLEARING CONNECTION POOL");
    while let Some(id) = cp_buffer_write.pop() {
        debug!("REMOVING: {}", &id);
        // Remove item from connection pool
        map_write.remove(&id);
        // Send strike message to lookup
        if let Some(addr) = pair.node.account.profile.lookup_address.clone() {
            lookup_strike::<T>(addr, id).await?;
        }
    }

    pair.sync.cp_size.fetch_sub(size, Ordering::SeqCst);
    Ok(())
}

/// Given an address and port, creates connection with new node
///
/// Function is passed an address and a port and it will attempt to
/// create a TCP connection with the node at that address
async fn create_connection<T: BlockChainBase>(
    pair: Arc<UserPair<T>>,
    address: String,
    connect_pool: Arc<ConnectionPool>,
) -> Result<()> {
    debug!("Creating Connection with: {}", address);
    // Open connection
    let mut stream: TcpStream = TcpStream::connect(address).await?;

    // Send initial message with ID
    let send_mess = NetworkMessage::<T>::new(MessageData::InitialID(
        pair.node.account.id,
        pair.node.account.pub_key.clone(),
        pair.node.account.role,
    ));
    send_message(&mut stream, send_mess).await?;

    // Recv initial message with ID
    let (id, pub_key, role) = match initial_stream_handler::<T>(&mut stream).await {
        Some((id, pub_key, role)) => (id, pub_key, role),
        _ => return Err(Error::msg("Error getting initial data Message")),
    };

    // Transmit initial bc state
    let bc_read = pair.node.blockchain.read().await;
    let bc_mess = NetworkMessage::<T>::new(MessageData::State(bc_read.clone()));
    send_message(&mut stream, bc_mess).await?;

    // Add to map
    match connect_pool
        .add(Connection::new(stream, role, Some(pub_key)), id)
        .await
    {
        Ok(_) => {
            pair.sync.cp_size.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Takes in [`NetworkMessage`] and sends it to all intended recipients
///
/// Gets a [`NetworkMessage`] and either floods all connections with the message
/// that is being sent, or will send it to all connected nodes
async fn send_all<T: BlockChainBase>(
    pair: Arc<UserPair<T>>,
    message: NetworkMessage<T>,
    connect_pool: Arc<ConnectionPool>,
) -> Result<()> {
    debug!("Sending message to all connected participants");
    let conn_map = connect_pool.map.read().await;
    for (id, conn) in conn_map.iter() {
        let tcp = conn.get_tcp();
        let mut stream = tcp.write.write().await;

        debug!("Sending message to: {:?}", tcp.addr);

        match send_message(stream.deref_mut(), message.clone()).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("ERROR SENDING TO: {}", id);
                pair.sync.cp_clear.store(true, Ordering::SeqCst);
                let mut cp_buffer_cpy = pair.sync.cp_buffer.write().await;
                cp_buffer_cpy.push(id.clone());
                Err(e)
            }
        }?;
    }
    Ok(())
}

/// Uses [`TcpListener`] to allow incoming connections
///
/// Listens to a inbound port and accepts connections coming from other
/// participants in the network. Takes the connections and inserts them
/// into the `connect_pool`.
async fn incoming_connections<T: 'static + BlockChainBase>(
    pair: Arc<UserPair<T>>,
    connect_pool: Arc<ConnectionPool>,
    listener: TcpListener,
) -> Result<()> {
    while let Ok((inbound, _)) = listener.accept().await {
        debug!("New Inbound Connection: {:?}", inbound.peer_addr());

        let fut = process_connection(inbound, Arc::clone(&pair), Arc::clone(&connect_pool));

        if let Err(e) = tokio::spawn(async move { fut.await }).await? {
            error!("Error processing connection: {}", e);
        }
    }

    Ok(())
}

/// Takes in a [`TcpStream`] and adds it to the [`ConnectionPool`]
///
/// Function is given a [`TcpStream`]. It then gets the ID from the
/// stream so the [`Connection`] can be identified in the [`ConnectionPool`].
async fn process_connection<T: 'static + BlockChainBase>(
    mut stream: TcpStream,
    pair: Arc<UserPair<T>>,
    connect_pool: Arc<ConnectionPool>,
) -> Result<()> {
    // Get the actual ID of the Connection from the stream
    let (id, pub_key, role) = match initial_stream_handler::<T>(&mut stream).await {
        Some((id, pub_key, role)) => (id, pub_key, role),
        _ => return Err(Error::msg("Error getting initial data Message")),
    };

    debug!("ID of new connection: {}", id);

    // Send back initial ID
    let send_mess = NetworkMessage::<T>::new(MessageData::InitialID(
        pair.node.account.id,
        pair.node.account.pub_key.clone(),
        pair.node.account.role,
    ));
    send_message(&mut stream, send_mess).await?;

    // Transmit initial bc state
    let bc_read = pair.node.blockchain.read().await;
    let bc_mess = NetworkMessage::<T>::new(MessageData::State(bc_read.clone()));
    send_message(&mut stream, bc_mess).await?;

    let conn = Connection::new(stream, role, Some(pub_key));
    match connect_pool.add(conn, id).await {
        Ok(_) => {
            pair.sync.cp_size.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Finds the [`MessageData::InitialID`] in a [`TcpStream`]
async fn initial_stream_handler<T: BlockChainBase>(
    stream: &mut TcpStream,
) -> Option<(u128, RsaPublicKey, Role)> {
    let mut buffer = [0_u8; 4096];
    match NetworkMessage::<T>::from_stream(stream, &mut buffer).await {
        Ok(m) => match m.data {
            MessageData::InitialID(id, key, role) => Some((id, key, role)),
            _ => None,
        },
        _ => None,
    }
}

/// Goes through each [`Connection`] and checks to see if they contain a [`NetworkMessage`]
///
/// Goes through all connections and checks if they have sent anything. If the stream is shutdown
/// then the [`Connection`] is removed from the [`ConnectionPool`].
async fn check_connections<T, F, Fut>(
    pair: Arc<UserPair<T>>,
    connect_pool: Arc<ConnectionPool>,
    state_machine: F,
) -> Result<()>
where
    T: 'static + BlockChainBase + std::marker::Sync,
    F: FnOnce(Arc<UserPair<T>>, NetworkMessage<T>) -> Fut + Send + 'static + Copy,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    let conns = connect_pool.map.read().await;

    for (id, conn) in conns.iter() {
        let pair_cpy = Arc::clone(&pair);
        let stream_lock: Arc<Halves> = conn.get_tcp();

        let pair_cp = Arc::clone(&pair);
        let id_cpy = id.clone();

        let _ret: Result<()> = tokio::spawn(async move {
            let mut stream = stream_lock.read.write().await;
            let mut buffer = [0_u8; 4096];

            // Check to see if connection has a message
            let peeked = match stream.peek(&mut buffer).await {
                Ok(p) => p,
                Err(_) => {
                    pair_cpy.sync.cp_clear.store(true, Ordering::SeqCst);
                    let mut cp_buffer_cpy = pair_cpy.sync.cp_buffer.write().await;
                    cp_buffer_cpy.push(id_cpy);
                    return Ok(());
                }
            };

            // If it does, pass to state machine
            if peeked > 0 {
                let message: NetworkMessage<T> =
                    NetworkMessage::from_stream(stream.deref_mut(), &mut buffer)
                        .await
                        .map_err(|_| Error::msg("Error with stream"))?;

                debug!("Received Message From Network: {:?}", &message);

                state_machine(pair_cp, message).await?;
            }
            // if not, move on
            Ok(())
        })
        .await?;
    }

    Ok(())
}

/// Forges new outgoing connections and adds them to the [`ConnectionPool`].
///
/// Consumes data from pipeline which instructs it to perform certain actions. This could be to
/// try and create a connection with another member of the network via a [`TcpStream`].
pub async fn outgoing_connections<T: 'static + BlockChainBase>(
    pair: Arc<UserPair<T>>,
    connect_pool: Arc<ConnectionPool>,
) -> Result<()> {
    let mut unsent_q: Vec<ProcessMessage<T>> = Vec::new();
    loop {
        // Check if there are any jobs in unsent_q
        if unsent_q.len() > 0 {
            debug!("Getting ProcessMessage from Queue");

            if let Some((i, ProcessMessage::NewConnection(_, addr))) = unsent_q
                .iter()
                .enumerate()
                .filter(|(_, m)| matches!(m, ProcessMessage::NewConnection(_, _)))
                .take(1)
                .next()
            {
                debug!("Forming connection with: {}", addr);

                create_connection::<T>(
                    Arc::clone(&pair),
                    String::from(addr),
                    Arc::clone(&connect_pool),
                )
                .await?;

                debug!("Removing element: {}", i);

                unsent_q.remove(i);
            }
        }

        let num_conns: usize = pair.sync.cp_size.load(Ordering::SeqCst);

        if num_conns == 0 && pair.node.account.profile.lookup_address.is_some() {
            match general_lookup(
                Arc::clone(&pair),
                pair.node.account.profile.lookup_address.clone().unwrap(),
            )
            .await
            {
                Ok(_) => (),
                Err(e) => {
                    error!("GENERAL LOOKUP FAILED: {}", e);
                    sleep(Duration::from_millis(10000)).await;
                }
            };
        }

        info!("CHECKING PERMIT");
        // Wait until there is something in the pipeline
        pair.sync.claim_permit().await;
        info!("CLAIMED PERMIT");

        let num_conns: usize = pair.sync.cp_size.load(Ordering::SeqCst);

        debug!("NUMBER OF CONNS: {}", num_conns);
        // Read pipeline for s
        let mut rx = pair.sync.outbound_channel.1.write().await;
        // When new message comes through pipeline
        if let Some(m) = rx.recv().await {
            debug!("Received message from pipeline: {:?}", &m);
            // Take message
            // Process message by reading using match to determine what to do
            // take action based on message

            match &m {
                ProcessMessage::SendMessage(net_mess) => {
                    // TODO: If Miner, and event is being sent, add it to block queue
                    if num_conns == 0 {
                        debug!("No connections, so adding to unsent queue");
                        unsent_q.push(m.clone());
                        continue;
                    }
                    match send_all(
                        Arc::clone(&pair),
                        net_mess.clone(),
                        Arc::clone(&connect_pool),
                    )
                    .await
                    {
                        Ok(_) => debug!("MESSAGE SENDING SUCCESS"),
                        Err(e) => error!("MESSAGE SENDING ERROR: {}", e),
                    };
                }
                ProcessMessage::NewConnection(id, addr) => {
                    match create_connection::<T>(
                        Arc::clone(&pair),
                        String::from(addr),
                        Arc::clone(&connect_pool),
                    )
                    .await
                    {
                        Ok(_) => debug!("CONNECTION CREATION SUCCESS"),
                        Err(e) => {
                            if let Some(lookup_addr) =
                                pair.node.account.profile.lookup_address.clone()
                            {
                                lookup_strike::<T>(lookup_addr, id.clone()).await?;
                            }
                            error!("CONNECTION CREATION FAILED: {}", e)
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}

/// Function to start up functions which deal with inbound
/// connections and message traffic
pub async fn inbound<T, F, Fut>(
    pair: Arc<UserPair<T>>,
    connect_pool: Arc<ConnectionPool>,
    listener: TcpListener,
    state_machine: F,
) -> Result<()>
where
    T: 'static + BlockChainBase + std::marker::Sync,
    F: FnOnce(Arc<UserPair<T>>, NetworkMessage<T>) -> Fut + Send + 'static + Copy,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    // Thread to listen for inbound connections (reactive)
    //     Put all connections into connect pool
    let pair_cpy = Arc::clone(&pair);
    let cp_cpy = Arc::clone(&connect_pool);
    tokio::spawn(async move {
        incoming_connections(pair_cpy, cp_cpy, listener)
            .await
            .unwrap()
    });
    // Thread to go through all connections and deal with incoming messages (reactive)
    let _conns: Result<()> = tokio::spawn(async move {
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
                    state_machine,
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

/// Function to replace the [`BlockChain`] in [`UserPair`]
pub async fn replace_blockchain<T: 'static + BlockChainBase + std::marker::Sync>(
    pair: Arc<UserPair<T>>,
    bc: &BlockChain<T>,
) -> Result<()> {
    info!("New blockchain received, old Blockchain replaced");
    let mut bc_unlocked = pair.node.blockchain.write().await;

    // Set new save blockchain location to one of previous blockchain
    let mut new_bc = bc.clone();
    new_bc.set_save_location(bc_unlocked.save_location());

    // Save new blockchain
    *bc_unlocked = new_bc;
    bc_unlocked.save()?;

    Ok(())
}

/// Function to add a new [`Block`] to [`BlockChain`]
pub async fn add_block<T: 'static + BlockChainBase + std::marker::Sync>(
    pair: Arc<UserPair<T>>,
    block: &Block<T>,
) -> Result<()> {
    pair.node
        .add_block(block.clone(), Arc::clone(&pair))
        .await?;
    // Drop events contained in block from loose_events
    let mut unlocked_loose = pair.node.loose_events.write().await;
    let dropped = unlocked_loose
        .drain_filter(|x| block.events.contains(x))
        .collect::<Vec<Event<T>>>();

    debug!("Events found to be in block: {:?}", dropped);

    // pass onto other connected nodes
    let message: NetworkMessage<T> = NetworkMessage::new(MessageData::Block(block.clone()));
    let process_message: ProcessMessage<T> = ProcessMessage::SendMessage(message);
    match pair.sync.outbound_channel.0.send(process_message) {
        Ok(_) => pair.sync.new_permit(),
        Err(e) => return Err(Error::msg(format!("Error writing block: {}", e))),
    };

    Ok(())
}

/// Main runner function for participant functionality
///
/// Needs to have a defined base datatype for the [`BlockChain`] to
/// store. The default one in this library is [`Data`]. Also provided
/// is the ability to pass in a function which can interact with the
/// [`JobSync`] mechanism. This lets the user build application
/// logic which can be used to do specific tasks, like create and
/// send messages to the outgoing thread.
pub async fn runner<T, F, Fut>(
    pair: Arc<UserPair<T>>,
    port: Option<u16>,
    state_machine: F,
) -> Result<()>
where
    T: 'static + BlockChainBase + std::marker::Sync,
    F: FnOnce(Arc<UserPair<T>>, NetworkMessage<T>) -> Fut + Send + 'static + Copy,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
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

    let inbound_fut = inbound(
        Arc::clone(&pair),
        Arc::clone(&connect_pool),
        listener,
        state_machine,
    );

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
