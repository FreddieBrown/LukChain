use crate::blockchain::events::Event;
use rand::prelude::*;
use rsa::{PaddingScheme, RsaPrivateKey, RsaPublicKey};

#[derive(Clone, Debug)]
pub struct Account {
    pub id: u128,
    pub role: Role,
    pub pub_key: RsaPublicKey,
    pub(crate) priv_key: RsaPrivateKey,
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
        }
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
}

#[derive(Clone, Debug)]
pub enum Role {
    User,
    Miner,
}
