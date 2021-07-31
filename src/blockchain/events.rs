use crypto::digest::Digest;
use crypto::sha3::Sha3;

#[derive(Clone, Debug)]
pub struct Event {
    pub data: Data,
    pub nonce: u128,
}

impl Event {
    /// Create new `Event`
    pub fn new(data: Data) -> Self {
        Self { data, nonce: 0 }
    }

    /// Calculate hash of the `Event`
    pub fn calculate_hash(&self) -> Vec<u8> {
        let mut hasher = Sha3::sha3_256();
        let event_as_string = format!("{:?}", (&self.data, &self.nonce));
        hasher.input_str(&event_as_string);
        return Vec::from(hasher.result_str().as_bytes());
    }

    pub fn execute(&self) {
        println!("{:?}", self.data);
    }
}

#[derive(Clone, Debug)]
pub enum Data {
    IndividualMessage(String),
    GroupMessage(String),
}
