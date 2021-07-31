use crypto::digest::Digest;
use crypto::sha3::Sha3;
use rsa::{PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Event {
    pub made_by: u128,
    pub data: Data,
    pub nonce: u128,
    signature: Option<Vec<u8>>,
}

impl Event {
    /// Create new `Event`
    pub fn new(made_by: u128, data: Data) -> Self {
        Self {
            made_by,
            data,
            nonce: 0,
            signature: None,
        }
    }

    /// Calculate hash of the `Event`
    pub fn calculate_hash(&self) -> Vec<u8> {
        let mut hasher = Sha3::sha3_256();
        let event_as_string = format!("{:?}", (&self.data, &self.nonce));
        hasher.input_str(&event_as_string);
        return Vec::from(hasher.result_str().as_bytes());
    }

    pub fn get_sign(&self) -> Option<Vec<u8>> {
        self.signature.clone()
    }

    pub fn execute(&self, pub_key: Option<&RsaPublicKey>) {
        println!("{:?}", self.data);
        if self.signature.is_some() {
            if self.verify_sign(pub_key.unwrap()) {
                println!("Message correctly signed");
            } else {
                println!("Message incorrectly signed");
            }
        }
    }

    pub fn sign(&mut self, priv_key: &RsaPrivateKey) {
        let bytes = self.calculate_hash();
        let padding = PaddingScheme::new_pkcs1v15_sign(None);
        let enc_data = priv_key
            .sign(padding, &bytes[..])
            .expect("failed to encrypt");

        self.signature = Some(enc_data);
    }

    pub fn verify_sign(&self, pub_key: &RsaPublicKey) -> bool {
        let hash = self.calculate_hash();
        if let Some(s) = self.get_sign() {
            let padding = PaddingScheme::new_pkcs1v15_sign(None);

            return match pub_key.verify(padding, &hash, &s) {
                Ok(_) => true,
                _ => false,
            };
        }
        true
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Data {
    IndividualMessage(String),
    GroupMessage(String),
}
