use blockchat::network::{run, Role};

use std::collections::HashSet;

use pico_args::Arguments;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref HASHSET: HashSet<&'static str> = {
        let mut m = HashSet::new();
        m.insert("info");
        m.insert("trace");
        m.insert("debug");
        m.insert("error");
        m.insert("warn");
        m
    };
}

const HELP: &str = "\
BlockChat
USAGE:
  blochat [OPTIONS] --log LEVEL [INPUT]
FLAGS:
  -h, --help            Prints help information
OPTIONS:
  --log LEVEL          Sets logging level
  --role ROLE          Sets the role of the user in the network
ARGS:
  <INPUT>
";

#[derive(Debug)]
struct AppArgs {
    log_level: Option<log::Level>,
    role: Option<Role>,
}

/// main program function
/// Decides if the run in specific mode, or to run local simulation
#[tokio::main]
async fn main() {
    let mut pargs = Arguments::from_env();

    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }

    let args = AppArgs {
        // Parses a required value that implements `FromStr`.
        // Returns an error if not present.
        log_level: pargs
            .opt_value_from_str("--log")
            .unwrap_or(Some(log::Level::Info)),
        // Parses an optional value that implements `FromStr`.
        role: pargs.opt_value_from_str("--role").unwrap(),
    };

    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1")
    }

    if std::env::var("RUST_LOG").is_err() {
        if let Some(l) = args.log_level {
            std::env::set_var("RUST_LOG", l.to_string())
        } else {
            std::env::set_var("RUST_LOG", "info")
        }
    }

    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    if let Some(role) = args.role {
        info!("Starting up as a ... {:?}", role);

        run(role).await.unwrap();
    }
}
