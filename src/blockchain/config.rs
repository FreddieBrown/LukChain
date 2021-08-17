use crate::blockchain::Role;

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub profiles: Vec<Profile>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Profile {
    pub block_size: Option<usize>,
    pub lookup_address: Option<String>,
    pub lookup_filter: Option<Role>,
    pub user_location: Option<String>,
    pub bc_location: Option<String>,
}

impl Profile {
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
