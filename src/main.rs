use blockchat::{
    accounts::{Account, Role},
    blockchain::{
        events::{Data, Event},
        Block, BlockChain,
    },
};

/// Function to perform simulated user action
fn run_user() {
    let mut account: Account = Account::new(Role::User);
    // Create event
    // Send to neighbours
    // Listen to connections and pass on messages
    // Repeat x3
}

/// Function to perform simulated mining action
fn run_miner() {
    let mut account: Account = Account::new(Role::Miner);
    // Listen for incoming events
    // When enough events received, build block
    // Add block to blockchain and distribute block
    // Repeat
}

/// Function to start up threads for simulation
fn simulation() {
    run_user();
    run_miner();
}

/// main program function
/// Decides if the run in specific mode, or to run local simulation
fn main() {
    println!("Welcome to BlockChat!");

    simulation();
}
