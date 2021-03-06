///! Module which defines the behaviour of the lookup node
use crate::{
    network::{
        accounts::Role,
        messages::{traits::ReadLengthPrefix, MessageData, NetworkMessage},
        send_message,
    },
    BlockChainBase,
};

use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use anyhow::Result;
use rand::seq::IteratorRandom;
use rand::thread_rng;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tracing::{debug, error, info};

type AddressTable = Arc<RwLock<HashMap<u128, (String, Role, AtomicUsize)>>>;

/// Starts up lookup server functionality
///
/// A lookup server uses a Distributed Hash Table-like functionality to store
/// addresses of participants in the network, and allows connected network
/// participants to find nodes to form initial connections with.
pub async fn lookup_run<T: 'static + BlockChainBase>(port: Option<u16>) -> Result<()> {
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

    info!("LookUp Address: {}", &socket);

    while let Ok((inbound, _)) = listener.accept().await {
        debug!("New Inbound Connection: {:?}", inbound.peer_addr());

        let at_cp = Arc::clone(&address_table);

        let fut = process_lookup::<T>(inbound, at_cp);

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
async fn process_lookup<T: 'static + BlockChainBase>(
    mut stream: TcpStream,
    address_table: AddressTable,
) -> Result<()> {
    let mut buffer = [0_u8; 4096];

    loop {
        let addr_clone = Arc::clone(&address_table);
        let recv_message: NetworkMessage<T> =
            NetworkMessage::from_stream(&mut stream, &mut buffer).await?;
        // Deal with initial message
        // Either specific request, or general request

        let send_mess = NetworkMessage::<T>::new(match recv_message.data {
            MessageData::LookUpReg(id, addr, role) => {
                // Deal with data from message
                let mut unlocked_table = addr_clone.write().await;

                unlocked_table.insert(id, (addr, role, AtomicUsize::new(0)));
                debug!("Address Table: {:?}", &unlocked_table);
                MessageData::Confirm
            }
            MessageData::RequestAddress(id) => get_connection(id, addr_clone).await,
            MessageData::GeneralAddrRequest(id, r) => {
                let connections = get_connections(id, addr_clone, r).await;
                if connections.len() == 0 {
                    MessageData::NoAddr
                } else {
                    MessageData::PeerAddresses(connections)
                }
            }
            MessageData::Finish => {
                info!("Closing Connection with node");
                return Ok(());
            }
            MessageData::Strike(id) => {
                info!("Giving {} a strike", &id);
                return strike(id, addr_clone).await;
            }
            _ => MessageData::NoAddr,
        });
        send_message(&mut stream, &send_mess).await?;
    }
}

async fn get_connections(
    id: u128,
    address_table: AddressTable,
    role: Option<Role>,
) -> Vec<(u128, String)> {
    let unlocked_table = address_table.read().await;

    // Use filters etc to pick 4 random addresses from table
    unlocked_table
        .keys()
        .filter(|k| {
            *k != &id && {
                if let Some(r) = role {
                    debug!("{:?}", unlocked_table.get(k).unwrap().1 == r);
                    unlocked_table.get(k).unwrap().1 == r
                } else {
                    true
                }
            }
        })
        .map(|k| (k.clone(), unlocked_table.get(k).unwrap().0.clone()))
        .choose_multiple(&mut thread_rng(), 4)
}

/// Strike system for poorly behaving nodes
///
/// Adds strikes to connections which aren't interacting
/// with other nodes correctly. More than a certain number of
/// strikes and they are removed from the AddressTable.
async fn strike(id: u128, address_table: AddressTable) -> Result<()> {
    let mut unlocked_table = address_table.write().await;
    // Go through iterator and add strikes
    let datas: Vec<(u128, usize)> = unlocked_table
        .iter()
        .filter(|(k, _)| *k == &id)
        .map(|(k, (_, _, strike))| {
            strike.fetch_add(1, Ordering::SeqCst);
            (k.clone(), strike.load(Ordering::SeqCst))
        })
        .collect();

    for (key, size) in datas {
        assert!(size > 0);
        if size > 10 {
            unlocked_table.remove(&key);
            info!("Strike Limit Reached: Node {}", &key);
        }
    }
    // If too many strikes, remove from address table
    Ok(())
}

async fn get_connection<T: 'static + BlockChainBase>(
    id: u128,
    address_table: AddressTable,
) -> MessageData<T> {
    let unlocked_table = address_table.read().await;
    if let Some((addr, _, _)) = unlocked_table.get(&id) {
        MessageData::<T>::PeerAddress((id, addr.clone()))
    } else {
        MessageData::NoAddr
    }
}
