///! Runner functions for participating in network
use crate::config::Profile;
use crate::network::{
    accounts::Role,
    connections::ConnectionPool,
    lookup,
    messages::{NetworkMessage, ProcessMessage},
    nodes::Node,
    participants,
};

use std::collections::HashSet;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use anyhow::Result;

use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    Notify, RwLock,
};
use tracing::{debug, info};

/// Synchronisation struct to maintain the job queue
pub struct JobSync {
    pub nonce_set: RwLock<HashSet<u128>>,
    pub permits: AtomicUsize,
    pub waiters: AtomicUsize,
    pub notify: Notify,
    pub sender: Sender<ProcessMessage>,
    pub receiver: RwLock<Receiver<ProcessMessage>>,
}

impl JobSync {
    /// Creates a new instance of JobSync
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<ProcessMessage>(1000);
        Self {
            nonce_set: RwLock::new(HashSet::new()),
            permits: AtomicUsize::new(0),
            waiters: AtomicUsize::new(0),
            notify: Notify::new(),
            sender: tx,
            receiver: RwLock::new(rx),
        }
    }

    /// Adds a new permit to the synchronisation mechanism
    pub fn new_permit(&self) {
        debug!("Creating new permit");

        let waiters: usize = self.waiters.load(Ordering::SeqCst);
        self.permits.fetch_add(1, Ordering::SeqCst);

        if waiters >= 1 {
            debug!("Notifing");
            self.notify.notify_one();
        }
    }

    /// Claims a permit from the group of permits
    ///
    /// Claims a permit if one exists, if no permits exist, it will
    /// wait until a permit is available.
    pub async fn claim_permit(&self) {
        debug!("Claiming permit");
        let permits: usize = self.permits.load(Ordering::SeqCst);
        if permits == 0 {
            self.waiters.fetch_add(1, Ordering::SeqCst);
            debug!("No available permits, Waiting for permit");
            self.notify.notified().await;
            let permits: usize = self.permits.load(Ordering::SeqCst);
            debug!("Permit now available: {}", permits);
            self.waiters.fetch_sub(1, Ordering::SeqCst);
        }
        self.permits.fetch_sub(1, Ordering::SeqCst);
        debug!(
            "Claimed permit. Permits left: {}",
            self.permits.load(Ordering::SeqCst)
        );
    }
}

pub async fn run(role: Role, profile: Profile) -> Result<()> {
    info!("Input Profile: {:?}", &profile);

    let node: Arc<Node> = Arc::new(Node::new(role, profile.clone()));
    let connect_pool: Arc<ConnectionPool> = Arc::new(ConnectionPool::new());
    let sync: Arc<JobSync> = Arc::new(JobSync::new());

    match role {
        Role::LookUp => {
            // Start Lookup server functionality
            lookup::run(Some(8181)).await
        }
        _ => {
            participants::run(
                Arc::clone(&node),
                Arc::clone(&connect_pool),
                Arc::clone(&sync),
                profile,
                None,
            )
            .await
        }
    }
}

/// Sends a new message to a [`Connection`] in [`ConnectionPool`]
///
/// Takes in a new [`NetworkMessage`] and distributes it to a [`Connection`] in the
/// [`ConnectionPool`] so they are aware of the information which is bein spread.
pub async fn send_message(stream: &mut TcpStream, message: NetworkMessage) -> Result<()> {
    debug!("Sending Message: {:?}", &message);
    let bytes_message = message.as_bytes();
    stream.write_all(&bytes_message).await?;
    Ok(())
}
