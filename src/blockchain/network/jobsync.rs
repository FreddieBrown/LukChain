use crate::blockchain::{
    network::messages::{NetworkMessage, ProcessMessage},
    BlockChainBase,
};

use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    Notify, RwLock,
};
use tracing::debug;

/// Synchronisation struct to maintain the job queue
pub struct JobSync<T: BlockChainBase> {
    pub nonce_set: RwLock<HashSet<u128>>,
    pub permits: AtomicUsize,
    pub waiters: AtomicUsize,
    pub notify: Notify,
    pub sender: Sender<ProcessMessage<T>>,
    pub receiver: RwLock<Receiver<ProcessMessage<T>>>,
    pub write_back: RwLock<Vec<NetworkMessage<T>>>,
    pub write_permission: bool,
    pub write_notify: Notify,
}

impl<T: BlockChainBase> JobSync<T> {
    /// Creates a new instance of JobSync
    pub fn new(write_permission: bool) -> Self {
        let (tx, rx) = mpsc::channel::<ProcessMessage<T>>(1000);
        Self {
            nonce_set: RwLock::new(HashSet::new()),
            permits: AtomicUsize::new(0),
            waiters: AtomicUsize::new(0),
            notify: Notify::new(),
            sender: tx,
            receiver: RwLock::new(rx),
            write_back: RwLock::new(Vec::new()),
            write_permission,
            write_notify: Notify::new(),
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
