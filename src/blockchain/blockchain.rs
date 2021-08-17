use crate::blockchain::{events::Event, BlockChainBase, UserPair};

use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use anyhow::{Error, Result};
use crypto::digest::Digest;
use crypto::sha3::Sha3;
use rand::prelude::*;
use rsa::RsaPublicKey;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
/// Struct to define local copy of blockchain
pub struct BlockChain<T> {
    /// Contains all [`Block`] instances in chain
    pub chain: Vec<Block<T>>,
    /// All known participants in network
    pub users: HashMap<u128, RsaPublicKey>,
    /// Time since Epoch that [`BlockChain`] was created
    pub created_at: Duration,
    pending_events: Vec<Event<T>>,
    #[serde(skip)]
    blc_location: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
/// Encapsulation of groups of events mined and stored on [`BlockChain`]
pub struct Block<T> {
    /// Events contained within [`Block`]
    pub events: Vec<Event<T>>,
    /// Hash of preceding [`Block`] in [`BlockChain`]
    pub prev_hash: Option<String>,
    /// Hash of [`Block`]
    pub hash: Option<String>,
    /// Random value for each instance
    pub nonce: u128,
    /// Time [`Block`] was created at since Epoch
    pub created_at: Duration,
}

impl<T: BlockChainBase> BlockChain<T> {
    /// Creates a new [`Blockchain`] instance
    pub fn new(blockchain_path: Option<String>) -> Self {
        let path: String = if let Some(pt) = blockchain_path {
            pt
        } else {
            String::from("./blockchain.bin")
        };

        let mut pathbuf = PathBuf::new();
        pathbuf.push(&path);

        if pathbuf.exists() {
            let mut file = File::open(&path).unwrap();
            let mut blc: Self = bincode::deserialize_from(&mut file).unwrap();
            blc.blc_location = Some(path);
            blc
        } else {
            let blc = Self {
                chain: Vec::new(),
                users: HashMap::new(),
                created_at: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap(),
                pending_events: Vec::new(),
                blc_location: Some(path.clone()),
            };

            // Write to [`BlockChain`] file
            #[cfg(not(test))]
            blc.save().unwrap();

            blc
        }
    }

    /// Gets the hash of the most recent [`Block`] in the chain
    pub fn last_hash(&self) -> Option<String> {
        if self.chain.len() == 0 {
            return None;
        }

        self.chain[self.chain.len() - 1].hash.clone()
    }

    /// Adds a new [`Block`] to the [`Blockchain`]
    ///
    /// [`id`] is the ID of the user who is in charge of the local instance of the
    /// [`BlockChain`]
    pub async fn append(&mut self, block: Block<T>, pair: Arc<UserPair<T>>) -> Result<()> {
        // Validate [`Block`]
        if !block.verify_hash() {
            return Err(Error::msg("Own hash could not be varified"));
        }

        if block.prev_hash != self.last_hash() {
            return Err(Error::msg("Invalid prev_hash stored in block"));
        }

        // Go through each event and check the data enclosed
        if block.execute(&self.users) {
            // Write back block
            pair.sync.write_block(block.clone()).await?;

            // If valid, append to [`BlockChain`] and return [`Ok`]
            self.chain.push(block);
        }

        Ok(())
    }

    /// Goes through the [`Blockchain`] and validates it
    pub fn validate_chain(&self) -> Result<()> {
        for (i, block) in self.chain.iter().enumerate() {
            if !block.verify_hash() {
                return Err(Error::msg("Own hash could not be varified"));
            }

            if i == 0 {
                if block.hash.is_some() {
                    return Err(Error::msg("Starting block shouldn't have prev_hash"));
                }
            } else {
                if !block.verify_hash() {
                    return Err(Error::msg("Own hash could not be varified"));
                }

                if block.prev_hash != self.chain[i - 1].hash {
                    return Err(Error::msg(format!(
                        "Invalid prev_hash stored in block {}",
                        i
                    )));
                }
            }

            // TODO: Go through and check correst signatures
        }
        Ok(())
    }

    /// Goes through the [`BlockChain`] and checks if [`Event`]
    /// is already in [`BlockChain`]
    pub fn contains(&self, event: &Event<T>) -> bool {
        self.chain
            .iter()
            .fold(false, |a, b| (b.events.contains(event)) || a)
    }

    /// Adds a new user to [`BlockChain`]
    pub fn new_user(&mut self, id: u128, pub_key: RsaPublicKey) {
        // TODO: In future generate user id and return it
        self.users.insert(id, pub_key);
    }

    /// Length of underlying [`BlockChain`]
    pub fn len(&self) -> usize {
        self.chain.len()
    }

    /// Calculates the percentage similarity with compared [`BlockChain`]
    pub fn chain_overlap(&self, chain: &BlockChain<T>) -> f64 {
        let mut counter = 0;
        for (base, comp) in self.chain.iter().zip(chain.chain.iter()) {
            if base == comp {
                counter += 1;
            } else {
                break;
            }
        }
        (counter as f64) / (self.len() as f64)
    }

    /// Check if [`Block`] is in chain
    pub fn in_chain(&self, block: &Block<T>) -> bool {
        self.chain.iter().filter(|b| b == &block).count() > 0
    }

    /// Sets the location to write the [`BlockChain`] to
    pub fn set_save_location(&mut self, location: Option<String>) {
        self.blc_location = location;
    }

    /// Returns the location of the  [`BlockChain`] in the filesystem
    pub fn save_location(&self) -> Option<String> {
        self.blc_location.clone()
    }

    /// Writes the current state of the [`BlockChain`] to its specified file
    pub fn save(&self) -> Result<()> {
        if let Some(loc) = self.blc_location.clone() {
            let mut file = File::create(loc)?;

            bincode::serialize_into(&mut file, self)?;
        }
        Ok(())
    }
}

impl<T: BlockChainBase> Block<T> {
    /// Creates a new [`Block`]
    pub fn new(prev_hash: Option<String>) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            events: Vec::new(),
            prev_hash,
            hash: None,
            nonce: rng.gen(),
            created_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap(),
        }
    }

    /// Adds multiple events to [`Block`]
    pub fn add_events(&mut self, events: Vec<Event<T>>) {
        self.events = events;
        self.hash = Some(self.calculate_hash());
    }

    /// Sets the nonce of the [`Block`]
    pub fn set_nonce(&mut self, nonce: u128) {
        self.nonce = nonce;
        self.update_hash();
    }

    // Cryptographic functions

    /// Calculates the hash of a given [`Block`]
    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha3::sha3_256();

        for event in self.events.iter() {
            hasher.input(&event.calculate_hash())
        }

        let block_as_string = format!("{:?}", (&self.prev_hash, &self.nonce));
        hasher.input_str(&block_as_string);

        return hasher.result_str();
    }

    /// Updates the hash of a given [`Block`]
    pub fn update_hash(&mut self) {
        self.hash = Some(self.calculate_hash());
    }

    /// Verifies the hash of the [`Block`]
    pub fn verify_hash(&self) -> bool {
        self.hash.is_some() && self.hash.as_ref().unwrap().eq(&self.calculate_hash())
    }

    // Functions for events

    /// Adds a event to a [`Block`]
    pub fn add_event(&mut self, event: Event<T>) {
        self.events.push(event);
        self.update_hash();
    }

    /// Gets the number of events in a given [`Block`]
    pub fn get_event_count(&self) -> usize {
        self.events.len()
    }

    /// Goes through each [`Event`] in [`Block`], checks validity and
    /// information each contains
    pub fn execute(&self, users: &HashMap<u128, RsaPublicKey>) -> bool {
        // TODO: Check for nonces that have already been used
        self.events.iter().fold(true, |acc, event| {
            acc && {
                if users.contains_key(&event.made_by) {
                    event.execute(users.get(&event.made_by))
                } else {
                    event.execute(None)
                }
            }
        })
    }
}
