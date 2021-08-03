///! Runner functions for participating in network
use crate::network::{
    accounts::Role,
    connections::{Connection, ConnectionPool},
    network_message::{MessageData, NetworkMessage},
    nodes::Node,
};

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;

use anyhow::{Error, Result};
use std::ops::DerefMut;
use tokio::net::{TcpListener, TcpStream};
use tokio::task;

pub async fn run(role: Role) -> Result<()> {
    let node: Arc<Node> = Arc::new(Node::new(role));
    let connect_pool: Arc<ConnectionPool> = Arc::new(ConnectionPool::new());
    inbound(Arc::clone(&node), Arc::clone(&connect_pool))
        .await
        .unwrap();
    outbound(Arc::clone(&node), Arc::clone(&connect_pool))
        .await
        .unwrap();
    Ok(())
}

async fn inbound(node: Arc<Node>, connect_pool: Arc<ConnectionPool>) -> Result<()> {
    // Thread to listen for inbound connections (reactive)
    //     Put all connections into connect pool
    let cp_copy = Arc::clone(&connect_pool);
    task::spawn(async move { incoming_connections(cp_copy).await.unwrap() }).await?;
    // Thread to go through all connections and deal with incoming messages (reactive)
    let node_copy = Arc::clone(&node);
    let cp_copy = Arc::clone(&connect_pool);
    task::spawn(async move { check_connections(node_copy, cp_copy).await.unwrap() }).await?;

    Ok(())
}

async fn outbound(node: Arc<Node>, connect_pool: Arc<ConnectionPool>) -> Result<()> {
    // Thread to forge outgoing connections and send outgoing messages (proactive)
    //     Put all connections into connect pool
    task::spawn(async move { outgoing_connections(node, connect_pool).await.unwrap() }).await?;

    Ok(())
}

/// Uses [`TcpListener`] to allow incoming connections
///
/// Listens to a inbound port and accepts connections coming from other
/// participants in the network. Takes the connections and inserts them
/// into the `connect_pool`.
async fn incoming_connections(connect_pool: Arc<ConnectionPool>) -> Result<()> {
    #[cfg(not(debug_assertions))]
    let ip = Ipv4Addr::UNSPECIFIED;

    #[cfg(debug_assertions)]
    let ip = Ipv4Addr::LOCALHOST;

    // Temporary solution
    let port: u16 = 8080;

    // Open socket and start listening
    let socket = SocketAddr::V4(SocketAddrV4::new(ip, port));
    let listener = TcpListener::bind(&socket).await?;

    while let Ok((inbound, _)) = listener.accept().await {
        let cp_clone = Arc::clone(&connect_pool);

        let fut = process_connection(inbound, cp_clone);

        if let Err(e) = tokio::spawn(async move { fut.await }).await? {
            println!("Error processing connection: {}", e);
        }
    }

    Ok(())
}

/// Takes in a [`TcpStream`] and adds it to the [`ConnectionPool`]
///
/// Function is given a [`TcpStream`]. It then gets the ID from the
/// stream so the [`Connection`] can be identified in the [`ConnectionPool`].
async fn process_connection(
    mut stream: TcpStream,
    connect_pool: Arc<ConnectionPool>,
) -> Result<()> {
    // Get the actual ID of the Connection from the stream
    let id: u128 = match initial_stream_handler(&mut stream).await {
        Some(i) => i,
        _ => return Err(Error::msg("Error getting initial ID Message")),
    };

    let conn = Connection::new(stream);
    connect_pool.add(conn, id).await;

    Ok(())
}

/// Finds the [`MessageData::InitialID`] in a [`TcpStream`]
async fn initial_stream_handler(stream: &mut TcpStream) -> Option<u128> {
    let mut buffer = [0_u8; 4096];
    match NetworkMessage::from_stream(stream, &mut buffer).await {
        Ok(m) => match m.data {
            MessageData::InitialID(id) => Some(id),
            _ => None,
        },
        _ => None,
    }
}

/// Goes through each [`Connection`] and checks to see if they contain a [`NetworkMessage`]
async fn check_connections(node: Arc<Node>, connect_pool: Arc<ConnectionPool>) -> Result<()> {
    let conns = connect_pool.map.read().await;

    for (_, conn) in conns.iter() {
        let stream_lock = conn.get_tcp();

        let node_cp = Arc::clone(&node);

        task::spawn(async move {
            let mut stream = stream_lock.write().await;
            let mut buffer = [0_u8; 4096];

            // Check to see if connection has a message
            let peeked = stream.peek(&mut buffer).await.unwrap();

            // If it does, pass to state machine
            if peeked > 0 {
                let message: NetworkMessage =
                    NetworkMessage::from_stream(stream.deref_mut(), &mut buffer)
                        .await
                        .map_err(|_| Error::msg("Error with stream"))
                        .unwrap();

                state_machine(node_cp, stream.deref_mut(), message)
                    .await
                    .unwrap();
            }
            // if not, move on
        });
    }
    Ok(())
}

/// Deals with incoming messages from each [`Connection`] in the [`ConnectionPool`]
///
/// Listens to each [`Connection`] and consumes any messages from the associated [`TcpStream`].
/// This message is then dealt with. Each [`NetworkMessage`] is processed using a state machine
/// structure, which is best suited to the unpredictable nature of the incoming messages.
async fn state_machine(
    node: Arc<Node>,
    stream: &mut TcpStream,
    message: NetworkMessage,
) -> Result<()> {
    Ok(())
}

/// Forges new outgoing connections and adds them to the [`ConnectionPool`].
///
/// Consumes data from pipeline which instructs it to perform certain actions. This could be to
/// try and create a connection with another member of the network via a [`TcpStream`].
async fn outgoing_connections(node: Arc<Node>, connect_pool: Arc<ConnectionPool>) -> Result<()> {
    Ok(())
}

/// Sends a new message to a [`Connection`] in [`ConnectionPool`]
///
/// Takes in a new [`NetworkMessage`] and distributes it to a [`Connection`] in the
/// [`ConnectionPool`] so they are aware of the information which is bein spread.
async fn send_message(
    node: Arc<Node>,
    stream: &mut TcpStream,
    message: NetworkMessage,
) -> Result<()> {
    Ok(())
}
