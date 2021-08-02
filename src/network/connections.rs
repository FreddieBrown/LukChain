use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct ConnectionPool {
    pub map: RwLock<HashMap<u128, Connection>>,
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub alive: bool,
    pub conn: Arc<RwLock<TcpStream>>,
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
    pub async fn add(&self, connection: Connection, id: u128) {
        let mut map = self.map.write().await;
        map.insert(id, connection);
    }

    /// Gets [`TcpStream`] reference based on its id
    ///
    /// Function is used to choose the next Connection to use.
    pub async fn get(&self, id: u128) -> Option<Arc<RwLock<TcpStream>>> {
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
        let stream = match self.get(id).await {
            Some(s) => s,
            None => return None,
        };
        // Unlock Stream
        let unlocked = stream.read().await;
        match unlocked.peer_addr() {
            Ok(addr) => Some(addr),
            _ => None,
        }
    }
}

impl Connection {
    /// Creates a new [`Connection`] object
    pub fn new(conn: TcpStream) -> Self {
        Self {
            conn: Arc::new(RwLock::new(conn)),
            alive: true,
        }
    }

    /// Gets [`TcpStream`] access
    ///
    /// Returns an Arc reference to a [`TcpStream`]. This is so access to the [`TcpStream`] can be
    /// acheived over multiple threads.
    pub fn get_tcp(&self) -> Arc<RwLock<TcpStream>> {
        Arc::clone(&self.conn)
    }
}
