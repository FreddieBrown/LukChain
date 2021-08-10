use blockchat::blockchain::{
    config::{Config, Profile},
    network::{lookup_run, participants_run, Role},
    Data,
};

use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;

use anyhow::Result;
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
  --config NUMBER      Sets chosen config (default: 0)
ARGS:
  <INPUT>
";

#[derive(Debug)]
struct AppArgs {
    log_level: Option<log::Level>,
    role: Option<Role>,
    config: Option<usize>,
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
        log_level: pargs.opt_value_from_str("--log").unwrap(),
        // Parses an optional value that implements `FromStr`.
        role: pargs.opt_value_from_str("--role").unwrap(),
        config: pargs.opt_value_from_str("--config").unwrap(),
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

    // Config in
    let mut input = String::new();
    File::open("config.toml")
        .and_then(|mut f| f.read_to_string(&mut input))
        .unwrap();

    let decoded: Config = toml::from_str(&input).unwrap();

    let config_size: usize = decoded.profiles.len();

    let chosen_profile: usize = args
        .config
        .map_or(0, |c| if c < config_size { c } else { 0 });

    let profile: Profile = decoded.profiles.get(chosen_profile).unwrap().to_owned();

    let chosen_role: Role = args.role.map_or(Role::User, |l| l);
    info!("Starting up as a ... {:?}", chosen_role);
    run(chosen_role, profile).await.unwrap();
}

pub async fn run(role: Role, profile: Profile) -> Result<()> {
    info!("Input Profile: {:?}", &profile);

    match role {
        Role::LookUp => {
            // Start Lookup server functionality
            lookup_run::<Data>(Some(8181)).await
        }
        _ => {
            participants_run::<Data>(
                profile,
                None,
                role,
                true,
                Some(|x| loop {
                    futures::executor::block_on(x.write_notify.notified());
                    {
                        let mut unlocked_read = futures::executor::block_on(x.write_back.write());

                        while unlocked_read.len() > 0 {
                            let message = unlocked_read.remove(0);
                            println!("MESSAGE: {:?}", message);
                        }
                    }
                }),
            )
            .await
        }
    }
}
