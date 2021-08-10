use rsa::{RsaPrivateKey, RsaPublicKey};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub profiles: Vec<Profile>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Profile {
    pub pub_key: Option<RsaPublicKey>,
    pub priv_key: Option<RsaPrivateKey>,
    pub block_size: Option<usize>,
    pub lookup_address: Option<String>,
}

impl Profile {
    pub fn new(
        pub_key: Option<RsaPublicKey>,
        priv_key: Option<RsaPrivateKey>,
        block_size: Option<usize>,
        lookup_address: Option<String>,
    ) -> Self {
        Self {
            pub_key,
            priv_key,
            block_size,
            lookup_address,
        }
    }
}
