pub mod events;

use anyhow::{Error, Result};
use crypto::digest::Digest;
use crypto::sha3::Sha3;

use events::Event;

#[derive(Clone, Debug)]
pub struct BlockChain {
    pub chain: Vec<Block>,
    pending_events: Vec<Event>,
}

impl BlockChain {
    /// Creates a new `Blockchain` instance
    pub fn new() -> Self {
        Self {
            chain: Vec::new(),
            pending_events: Vec::new(),
        }
    }

    /// Gets the hash of the most recent `Block` in the chain
    pub fn last_hash(&self) -> Option<String> {
        if self.chain.len() == 0 {
            return None;
        }

        self.chain[self.chain.len() - 1].hash.clone()
    }

    /// Adds a new `Block` to the `Blockchain`
    pub fn append(&mut self, block: Block) -> Result<()> {
        // Validate `Block`
        if !block.verify_hash() {
            return Err(Error::msg("Own hash could not be varified"));
        }

        if block.prev_hash != self.last_hash() {
            return Err(Error::msg("Invalid prev_hash stored in block"));
        }

        // TODO: Check for nonces that have already been used

        // TODO: Do something with each event in the Block
        for event in block.events.iter() {
            event.execute();
        }

        // If valid, append to `Blockchain` and return Ok
        self.chain.push(block);

        Ok(())
    }

    /// Goes through the `Blockchain` and validates it
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
}

#[derive(Clone, Debug)]
pub struct Block {
    pub events: Vec<Event>,
    pub prev_hash: Option<String>,
    pub hash: Option<String>,
    pub nonce: u128,
}

impl Block {
    /// Creates a new `Block`
    pub fn new(prev_hash: Option<String>) -> Self {
        Self {
            events: Vec::new(),
            prev_hash,
            hash: None,
            nonce: 0,
        }
    }

    /// Sets the nonce of the `Block`
    pub fn set_nonce(&mut self, nonce: u128) {
        self.nonce = nonce;
        self.update_hash();
    }

    // Cryptographic functions

    /// Calculates the hash of a given `Block`
    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha3::sha3_256();

        for event in self.events.iter() {
            hasher.input(&event.calculate_hash())
        }

        let block_as_string = format!("{:?}", (&self.prev_hash, &self.nonce));
        hasher.input_str(&block_as_string);

        return hasher.result_str();
    }

    /// Updates the hash of a given `Block`
    pub fn update_hash(&mut self) {
        self.hash = Some(self.calculate_hash());
    }

    /// Verifies the hash of the `Block`
    pub fn verify_hash(&self) -> bool {
        self.hash.is_some() && self.hash.as_ref().unwrap().eq(&self.calculate_hash())
    }

    // Functions for events

    /// Adds a event to a `Block`
    pub fn add_event(&mut self, event: Event) {
        self.events.push(event);
        self.update_hash();
    }

    /// Gets the number of events in a given `Block`
    pub fn get_event_count(&self) -> usize {
        self.events.len()
    }
}
