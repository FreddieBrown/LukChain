use rsa::{RsaPrivateKey, RsaPublicKey};

pub struct Account {
    pub id: u128,
    pub role: Role,
    pub pub_key: RsaPublicKey,
    pub(crate) priv_key: RsaPrivateKey,
}

impl Account {
    pub fn new(role: Role, pub_key: RsaPublicKey, priv_key: RsaPrivateKey) -> Self {
        Self {
            id: 0,
            role,
            pub_key,
            priv_key,
        }
    }
}

pub enum Role {
    User,
    Miner,
}
