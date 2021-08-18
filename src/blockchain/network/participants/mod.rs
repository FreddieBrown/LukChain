//! Functionality for different network roles

mod miners;
mod shared;
mod users;

pub use self::{miners::miners_run, users::users_run};
