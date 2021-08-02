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
    task::spawn(async move { message_state_machine(node_copy, cp_copy).await.unwrap() }).await?;

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

async fn message_state_machine(node: Arc<Node>, connect_pool: Arc<ConnectionPool>) -> Result<()> {
    Ok(())
}

async fn outgoing_connections(node: Arc<Node>, connect_pool: Arc<ConnectionPool>) -> Result<()> {
    Ok(())
}
