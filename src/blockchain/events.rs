//! Defining base element in the blockchain
use crate::blockchain::BlockChainBase;

use std::fmt::Debug;
use std::time::{Duration, SystemTime};

use crypto::digest::Digest;
use crypto::sha3::Sha3;
use rand::prelude::*;
use rsa::{PaddingScheme, PublicKey, RsaPublicKey};
use serde::{Deserialize, Serialize};

/// Events that are stored on [`BlockChain`]
///
/// When something needs to be stored on the [`BlockChain`], an
/// [`Event`] is created, and the data to be stored on the chain
/// is contained within it.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Event<T> {
    /// ID of node which created [`Event`]
    pub made_by: u128,
    /// Central information of [`Event`]
    pub data: T,
    /// Random value to ensure uniqueness
    pub nonce: u128,
    /// Mechanism to provide validity of [`Event`]
    pub signature: Option<Vec<u8>>,
    /// Time [`Event`] was created since Epoch
    pub created_at: Duration,
}

/// Basic struct to show what can be stored on the [`BlockChain`]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Data {
    /// Message to send to specific user. Data is encrypted
    IndividualMessage(u128, Vec<u8>),
    /// Message to be read by anyone
    GroupMessage(String),
    /// Adds the creation of a new node to the [`BlockChain`]
    NewUser {
        /// ID of new user
        id: u128,
        /// Public Key of new user
        pub_key: RsaPublicKey,
    },
}

impl<T: BlockChainBase> Event<T> {
    /// Create new `Event`
    pub fn new(made_by: u128, data: T) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            made_by,
            data,
            nonce: rng.gen(),
            signature: None,
            created_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap(),
        }
    }

    /// Calculate hash of the `Event`
    pub fn calculate_hash(&self) -> Vec<u8> {
        let mut hasher = Sha3::sha3_256();
        let event_as_string = format!("{:?}", (&self.data, &self.nonce));
        hasher.input_str(&event_as_string);
        return Vec::from(hasher.result_str().as_bytes());
    }

    /// Checks the validity of the [`Event`]
    pub fn execute(&self, foreign_pub_key: Option<&RsaPublicKey>) -> bool {
        if self.signature.is_some() {
            self.verify_sign(foreign_pub_key.unwrap())
        } else {
            true
        }
    }

    /// Adds signature to the [`Event`]
    pub fn sign(&mut self, signature: Option<Vec<u8>>) {
        self.signature = signature;
    }

    /// Verifies the associated signature in the [`Event`]
    pub fn verify_sign(&self, pub_key: &RsaPublicKey) -> bool {
        let hash = self.calculate_hash();
        if let Some(s) = self.signature.clone() {
            let padding = PaddingScheme::new_pkcs1v15_sign(None);

            return match pub_key.verify(padding, &hash, &s) {
                Ok(_) => true,
                _ => false,
            };
        }
        true
    }
}

impl BlockChainBase for Data {}
