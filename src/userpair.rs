pub use crate::{
    config::Profile,
    network::{JobSync, Node, Role},
    Block, BlockChain, BlockChainBase, Data, Event,
};

use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use anyhow::Result;
use rand::prelude::*;
use rsa::{RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};

/// Wrapper struct containing information needed for each user
#[derive(Debug)]
pub struct UserPair<T: BlockChainBase> {
    /// Synchronisation Struct for blockchain
    pub sync: JobSync<T>,
    /// Information about network node
    pub node: Node<T>,
}

impl<T: BlockChainBase> UserPair<T> {
    /// Creates a new instance of the [`UserPair`] struct
    ///
    /// Gets information stored for the participant or generates it.
    /// This is passed up and used to setup the [`UserPair`] on startup.
    pub async fn new(role: Role, profile: Profile, write_back: bool) -> Result<Self> {
        let pinfo: PersistentInformation =
            PersistentInformation::new(profile.user_location.clone());
        let sync = JobSync::new(write_back);
        let node: Node<T> = if matches!(role, Role::Miner) {
            Node::genesis(profile, &sync, pinfo).await?
        } else {
            Node::new(role, profile, pinfo)
        };

        Ok(Self { sync, node })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentInformation {
    pub pub_key: RsaPublicKey,
    pub priv_key: RsaPrivateKey,
    pub id: u128,
}

impl PersistentInformation {
    /// Check to see if file exists. If file exists, build from file
    /// If file doesn't exist, create key pair and write information to file
    pub fn new(user_data: Option<String>) -> Self {
        let mut path: PathBuf = PathBuf::new();
        path.push(if let Some(pt) = user_data {
            pt
        } else {
            String::from("./user.json")
        });

        if path.exists() {
            // Check if there is a file with the correct name
            let mut input = String::new();
            File::open(path)
                .and_then(|mut f| f.read_to_string(&mut input))
                .unwrap();

            let from_file: Self = serde_json::from_str(&input).unwrap();
            from_file
        } else {
            let mut rng = rand::thread_rng();
            let (pub_key, priv_key): (RsaPublicKey, RsaPrivateKey) = {
                let priv_key =
                    RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate a key");
                let pub_key = RsaPublicKey::from(&priv_key);
                (pub_key, priv_key)
            };

            let info = Self {
                pub_key,
                priv_key,
                id: rng.gen(),
            };
            // Write info to file
            let mut open_file = File::create(path).unwrap();
            let to_write = serde_json::to_vec(&info).unwrap();
            open_file.write_all(&to_write).unwrap();
            // Return info
            info
        }
    }
}
