use blockchat::network::{run, Role};

/// main program function
/// Decides if the run in specific mode, or to run local simulation
#[tokio::main]
async fn main() {
    run(Role::Miner).await.unwrap();
}
