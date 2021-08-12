use crate::blockchain::{network::messages::ProcessMessage, Block, BlockChainBase};

use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::{Error, Result};
use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    Notify, RwLock,
};
use tracing::debug;

/// Synchronisation struct to maintain the job queue
pub struct JobSync<T: BlockChainBase> {
    pub nonce_set: RwLock<HashSet<u128>>,
    pub permits: AtomicUsize,
    pub waiters: AtomicUsize,
    pub notify: Notify,
    pub outbound_channel: (
        UnboundedSender<ProcessMessage<T>>,
        RwLock<UnboundedReceiver<ProcessMessage<T>>>,
    ),
    pub app_channel: (
        UnboundedSender<Block<T>>,
        RwLock<UnboundedReceiver<Block<T>>>,
    ),
    pub app_permission: bool,
    pub app_notify: Notify,
}

impl<T: BlockChainBase> JobSync<T> {
    /// Creates a new instance of JobSync
    pub fn new(app_permission: bool) -> Self {
        let (ptx, prx) = mpsc::unbounded_channel::<ProcessMessage<T>>();
        let (wtx, wrx) = mpsc::unbounded_channel::<Block<T>>();
        Self {
            nonce_set: RwLock::new(HashSet::new()),
            permits: AtomicUsize::new(0),
            waiters: AtomicUsize::new(0),
            notify: Notify::new(),
            outbound_channel: (ptx, RwLock::new(prx)),
            app_channel: (wtx, RwLock::new(wrx)),
            app_permission,
            app_notify: Notify::new(),
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

    pub async fn write_block(&self, block: Block<T>) -> Result<()> {
        if self.app_permission {
            // let mut wb_unlocked = self.outbound_channel.0;
            match self.app_channel.0.send(block) {
                Ok(_) => self.app_notify.notify_one(),
                Err(e) => return Err(Error::msg(format!("Error writing block: {}", e))),
            };
        }
        Ok(())
    }
}
