//! Defining base element in the blockchain

use std::cmp::PartialEq;
use std::time::{Duration, SystemTime};

use crypto::digest::Digest;
use crypto::sha3::Sha3;
use rand::prelude::*;
use rsa::{PaddingScheme, PublicKey, RsaPublicKey};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Event {
    pub made_by: u128,
    pub data: Data,
    pub nonce: u128,
    pub signature: Option<Vec<u8>>,
    pub created_at: Duration,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Data {
    // Need to encrypt data in message using target public key
    IndividualMessage(u128, Vec<u8>),
    GroupMessage(String),
    NewUser { id: u128, pub_key: RsaPublicKey },
}

impl Event {
    /// Create new `Event`
    pub fn new(made_by: u128, data: Data) -> Self {
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

    pub fn execute(&self, pub_key: Option<&RsaPublicKey>) {
        match self.data.clone() {
            Data::IndividualMessage(id, _) => {
                println!("SENDER {} FOR {}: ENCRYPTED", self.made_by, id)
            }
            Data::GroupMessage(m) => println!("MESSAGE: {}", m),
            Data::NewUser { id, .. } => println!("NEW USER: {}", id),
        };

        if self.signature.is_some() {
            if self.verify_sign(pub_key.unwrap()) {
                println!("Message correctly signed");
            } else {
                println!("Message incorrectly signed");
            }
        }
    }

    pub fn sign(&mut self, signature: Option<Vec<u8>>) {
        self.signature = signature;
    }

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

impl Data {
    pub fn new_individual_message(location: u128, pub_key: &RsaPublicKey, message: String) -> Self {
        let mut rng = rand::thread_rng();
        let padding = PaddingScheme::new_pkcs1v15_encrypt();
        let encrypted = pub_key
            .encrypt(&mut rng, padding, message.as_bytes())
            .expect("Failed to encrypt individual message");
        Self::IndividualMessage(location, encrypted)
    }
}
