//! Serializable/Deserializable structs to read/write config details for node
use crate::blockchain::Role;

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
/// Overall struct containing all Profiles
pub struct Config {
    /// Vec containing all read profiles
    pub profiles: Vec<Profile>,
}

#[derive(Deserialize, Debug, Clone)]
/// Single struct containing user defined details of how to run node
pub struct Profile {
    /// Number of events for miner to include in [`Block`]
    pub block_size: Option<usize>,
    /// Address to connect to a LookUp node on
    pub lookup_address: Option<String>,
    /// Roles to filter by when communicating with LookUp node
    pub lookup_filter: Option<Role>,
    /// Location of JSON file defining user data
    pub user_location: Option<String>,
    /// Location of binary file containing local copy of [`BlockChain`]
    pub bc_location: Option<String>,
}

impl Profile {
    /// Creates a new instance of [`Profile`]
    pub fn new(
        block_size: Option<usize>,
        lookup_address: Option<String>,
        lookup_filter: Option<Role>,
        user_location: Option<String>,
        bc_location: Option<String>,
    ) -> Self {
        Self {
            block_size,
            lookup_address,
            lookup_filter,
            user_location,
            bc_location,
        }
    }
}
