//! Information about the participant needed for network participation

use crate::blockchain::{config::Profile, BlockChainBase, Event};

use std::str::FromStr;

use anyhow::{Error, Result};
use rand::prelude::*;
use rsa::{PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Clone, Debug)]
pub struct Account {
    pub id: u128,
    pub role: Role,
    pub block_size: usize,
    pub pub_key: RsaPublicKey,
    pub(crate) priv_key: RsaPrivateKey,
}

#[derive(Clone, Debug, Copy, Deserialize, Serialize)]
pub enum Role {
    User,
    Miner,
    LookUp,
}

impl Account {
    pub fn new(role: Role, profile: Profile) -> Self {
        let mut rng = rand::thread_rng();
        let pub_key: RsaPublicKey;
        let priv_key: RsaPrivateKey;

        let (pub_key, priv_key): (RsaPublicKey, RsaPrivateKey) =
            if profile.pub_key.is_some() && profile.priv_key.is_some() {
                (profile.pub_key.unwrap(), profile.priv_key.unwrap())
            } else {
                priv_key = RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate a key");
                pub_key = RsaPublicKey::from(&priv_key);
                (pub_key, priv_key)
            };

        let block_size: usize = if profile.block_size.is_some() {
            profile.block_size.unwrap()
        } else {
            10
        };

        debug!("Generated RSA Pair");

        Self {
            id: rng.gen(),
            role,
            block_size,
            pub_key,
            priv_key,
        }
    }

    pub fn new_event<T: BlockChainBase>(&self, data: T) -> Event<T> {
        debug!("New Event: {:?}", data);
        let mut event = Event::new(self.id, data);
        self.sign_event(&mut event);
        event
    }

    pub fn sign_event<T: BlockChainBase>(&self, event: &mut Event<T>) {
        debug!("Signing Event: {:?}", event);

        let bytes = event.calculate_hash();
        let padding = PaddingScheme::new_pkcs1v15_sign(None);
        let enc_data = self
            .priv_key
            .sign(padding, &bytes[..])
            .expect("failed to sign");

        event.sign(Some(enc_data));
    }

    pub fn decrypt_msg(&self, enc_data: Vec<u8>) -> String {
        debug!("Decrypting Message: {:?}", &enc_data);

        let padding = PaddingScheme::new_pkcs1v15_encrypt();
        let dec_data = self
            .priv_key
            .decrypt(padding, &enc_data)
            .expect("failed to decrypt");

        String::from_utf8(dec_data).unwrap()
    }

    pub fn encrypt_msg(&self, data: Vec<u8>) -> Vec<u8> {
        debug!("Encrypting Message: {:?}", &data);

        let mut rng = rand::thread_rng();
        let padding = PaddingScheme::new_pkcs1v15_encrypt();
        self.priv_key
            .encrypt(&mut rng, padding, &data)
            .expect("failed to encrypt")
    }
}

impl FromStr for Role {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let lower = s.to_lowercase();
        match &lower[..] {
            "miner" => Ok(Role::Miner),
            "user" => Ok(Role::User),
            "lookup" => Ok(Role::LookUp),
            _ => Err(Error::msg("Error in FromStr in Role")),
        }
    }
}
