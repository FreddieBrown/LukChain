use crate::blockchain::network::Role;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{Error, Result};
use rsa::RsaPublicKey;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct ConnectionPool {
    pub map: RwLock<HashMap<u128, Connection>>,
}

#[derive(Debug)]
pub struct Connection {
    pub stream: Arc<Halves>,
    pub alive: bool,
    pub role: Role,
    pub pub_key: Option<RsaPublicKey>,
}

#[derive(Debug)]
pub struct Halves {
    pub read: RwLock<OwnedReadHalf>,
    pub write: RwLock<OwnedWriteHalf>,
    pub addr: Option<SocketAddr>,
}

impl ConnectionPool {
    /// Returns a [`ConnectionPool`] instance
    pub fn new() -> Self {
        Self {
            map: RwLock::new(HashMap::new()),
        }
    }

    /// Adds new [`Connection`] to [`ConnectionPool`]
    ///
    /// Function will take in a new [`Connection`] and will add it to the HashMap
    /// behind the ID that the external [`Node`] provided
    pub async fn add(&self, connection: Connection, id: u128) -> Result<()> {
        let mut map = self.map.write().await;
        if !map.contains_key(&id) {
            map.insert(id, connection);
            Ok(())
        } else {
            Err(Error::msg("ALREADY IN CONNECTION POOL"))
        }
    }

    /// Gets [`TcpStream`] reference based on its id
    ///
    /// Function is used to choose the next Connection to use.
    pub async fn get(&self, id: u128) -> Option<Arc<Halves>> {
        let map_read = self.map.read().await;

        for (_, conn) in map_read.iter().filter(|(key, _)| *key == &id) {
            let stream = conn.get_tcp();
            return Some(stream);
        }

        None
    }

    /// Get remote address of ['TcpStream`]
    ///
    /// Returns the address of the [`Node`] that the [`TcpStream`] connects to
    pub async fn addr(&self, id: u128) -> Option<SocketAddr> {
        // Get stream
        match self.get(id).await {
            Some(s) => s.addr,
            None => None,
        }
    }
}

impl Connection {
    /// Creates a new [`Connection`] object
    pub fn new(conn: TcpStream, role: Role, pub_key: Option<RsaPublicKey>) -> Self {
        let stream = Halves::new(conn);
        Self {
            stream: Arc::new(stream),
            alive: true,
            role,
            pub_key,
        }
    }

    /// Gets [`OwnedReadHalf`] and  [`OwnedWriteHalf`]access
    ///
    /// Returns an Arc reference to both halves of [`TcpStream`]. This is so access to the stream
    /// can be provided to multiple threads. This is through [`Halves`].
    pub fn get_tcp(&self) -> Arc<Halves> {
        Arc::clone(&self.stream)
    }
}

impl Halves {
    pub fn new(stream: TcpStream) -> Self {
        let addr = match stream.peer_addr() {
            Ok(a) => Some(a),
            Err(_) => None,
        };
        let (read, write) = stream.into_split();
        Self {
            read: RwLock::new(read),
            write: RwLock::new(write),
            addr,
        }
    }
}
