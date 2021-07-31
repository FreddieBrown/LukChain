#[cfg(test)]
mod tests;

use crate::blockchain::{
    events::{Data, Event},
    BlockChain,
};

use rand::prelude::*;
use rsa::{PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey};

#[derive(Clone, Debug)]
pub struct Account {
    pub id: u128,
    pub role: Role,
    pub pub_key: RsaPublicKey,
    pub(crate) priv_key: RsaPrivateKey,
    pub(crate) bc: BlockChain,
}

#[derive(Clone, Debug)]
pub enum Role {
    User,
    Miner,
}

impl Account {
    pub fn new(role: Role) -> Self {
        let mut rng = rand::thread_rng();
        let priv_key = RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate a key");
        let pub_key = RsaPublicKey::from(&priv_key);

        Self {
            id: rng.gen(),
            role,
            pub_key,
            priv_key,
            bc: BlockChain::new(),
        }
    }

    fn update_blockchain(&mut self, new_bc: BlockChain) {
        self.bc = new_bc;
    }

    pub fn new_event(&self, data: Data) -> Event {
        let mut event = Event::new(self.id, data);
        self.sign_event(&mut event);
        event
    }

    pub fn sign_event(&self, event: &mut Event) {
        let bytes = event.calculate_hash();
        let padding = PaddingScheme::new_pkcs1v15_sign(None);
        let enc_data = self
            .priv_key
            .sign(padding, &bytes[..])
            .expect("failed to encrypt");

        event.sign(Some(enc_data));
    }

    pub fn decrypt_msg(&self, enc_data: Vec<u8>) -> String {
        let padding = PaddingScheme::new_pkcs1v15_encrypt();
        let dec_data = self
            .priv_key
            .decrypt(padding, &enc_data)
            .expect("failed to decrypt");

        String::from_utf8(dec_data).unwrap()
    }

    pub fn encrypt_msg(&self, data: Vec<u8>) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let padding = PaddingScheme::new_pkcs1v15_encrypt();
        self.priv_key
            .encrypt(&mut rng, padding, &data)
            .expect("failed to decrypt")
    }
}
