//! Traits needed to execute the functionality of the blockchain
use async_trait::async_trait;
use rsa::RsaPublicKey;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

#[async_trait]
pub trait BlockChainBase:
    PartialEq + Eq + Serialize + Debug + Clone + DeserializeOwned + Send + Unpin + std::marker::Sync
{
    fn execute(&self, pub_key: Option<&RsaPublicKey>);
}
