use blockchat::network::{run, Role};

use tracing::info;
use tracing_subscriber::EnvFilter;

/// main program function
/// Decides if the run in specific mode, or to run local simulation
#[tokio::main]
async fn main() {
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1")
    }

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }

    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let role: Role = Role::User;

    info!("Starting up as a ... {:?}", role);

    run(role).await.unwrap();
}
