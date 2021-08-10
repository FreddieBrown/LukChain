///! Module which defines the behaviour of the lookup node
use crate::network::{
    accounts::Role,
    messages::{MessageData, NetworkMessage},
    runner::send_message,
};

use std::collections::HashMap;
use std::net::Shutdown;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;

use anyhow::Result;
use rand::seq::IteratorRandom;
use rand::thread_rng;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tracing::{debug, error, info};

pub type AddressTable = Arc<RwLock<HashMap<u128, (String, Role)>>>;

/// Starts up lookup server functionality
///
/// A lookup server uses a Distributed Hash Table-like functionality to store
/// addresses of participants in the network, and allows connected network
/// participants to find nodes to form initial connections with.
pub async fn run(port: Option<u16>) -> Result<()> {
    let address_table: AddressTable = Arc::new(RwLock::new(HashMap::new()));

    #[cfg(not(debug_assertions))]
    let ip = Ipv4Addr::UNSPECIFIED;

    #[cfg(debug_assertions)]
    let ip = Ipv4Addr::LOCALHOST;

    // Open socket and start listening
    let socket = match port {
        Some(p) => SocketAddr::V4(SocketAddrV4::new(ip, p)),
        _ => SocketAddr::V4(SocketAddrV4::new(ip, 0)),
    };

    let listener = TcpListener::bind(&socket).await?;

    info!("Running on: {}", &socket);

    while let Ok((inbound, _)) = listener.accept().await {
        debug!("New Inbound Connection: {:?}", inbound.peer_addr());

        let at_cp = Arc::clone(&address_table);

        let fut = process_lookup(inbound, at_cp);

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
async fn process_lookup(mut stream: TcpStream, address_table: AddressTable) -> Result<()> {
    let mut buffer = [0_u8; 4096];
    let mut client_id = 0;

    loop {
        let addr_clone = Arc::clone(&address_table);
        let recv_message: NetworkMessage =
            NetworkMessage::from_stream(&mut stream, &mut buffer).await?;
        // Deal with initial message
        // Either specific request, or general request

        let send_mess = NetworkMessage::new(match recv_message.data {
            MessageData::LookUpReg(id, addr, role) => {
                // Deal with data from message
                let mut unlocked_table = addr_clone.write().await;
                // Add node to table
                client_id = id;

                unlocked_table.insert(id, (addr, role));
                debug!("Address Table: {:?}", &unlocked_table);
                MessageData::Confirm
            }
            MessageData::RequestAddress(id) => get_connection(id, addr_clone).await,
            MessageData::GeneralAddrRequest => {
                let connections = get_connections(client_id, addr_clone).await;
                if connections.len() == 0 {
                    MessageData::NoAddr
                } else {
                    MessageData::PeerAddresses(connections)
                }
            }
            MessageData::Finish => break,
            _ => MessageData::NoAddr,
        });
        send_message(&mut stream, send_mess).await?;
    }

    stream.into_std()?.shutdown(Shutdown::Both)?;

    Ok(())
}

async fn get_connections(id: u128, address_table: AddressTable) -> Vec<String> {
    let unlocked_table = address_table.read().await;

    // Use filters etc to pick 4 random addresses from table
    unlocked_table
        .keys()
        .filter(|k| *k != &id)
        .map(|k| unlocked_table.get(k).unwrap().0.clone())
        .choose_multiple(&mut thread_rng(), 4)
}

async fn get_connection(id: u128, address_table: AddressTable) -> MessageData {
    let unlocked_table = address_table.read().await;
    if let Some((addr, _)) = unlocked_table.get(&id) {
        MessageData::PeerAddress(addr.clone())
    } else {
        MessageData::NoAddr
    }
}
