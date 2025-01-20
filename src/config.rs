use clap::{Arg, Command};
use log::{debug, error, info, warn};
use serde::Deserialize;
use std::env;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: String,
    pub interval_secs: u64,
}

// Default configuration constants
const DEFAULT_SERVER: &str = "127.0.0.1:5000";
const DEFAULT_INTERVAL_SECS: u64 = 10;

/// Load configuration from command-line arguments, configuration file, or environment variables.
pub fn load_config() -> AppConfig {
    info!("Starting configuration loading process.");

    // Step 1: Parse command-line arguments using `clap`
    let matches = Command::new("Gilded-Sentinel-Debian")
        .version("0.1.0")
        .author("LunarLaurus")
        .about("Collects and sends sensor data")
        .arg(
            Arg::new("server")
                .long("server")
                .help("Server address (e.g., 127.0.0.1:5000)")
                .value_parser(clap::value_parser!(String)),
        )
        .arg(
            Arg::new("interval")
                .long("interval")
                .help("Interval in seconds between data collection")
                .value_parser(clap::value_parser!(u64)),
        )
        .arg(
            Arg::new("config")
                .long("config")
                .help("Path to the configuration file")
                .value_parser(clap::value_parser!(String)),
        )
        .get_matches();

    debug!("Command-line arguments parsed successfully.");

    // Step 2: Load configuration from file if provided
    let file_config = matches
        .get_one::<String>("config")
        .map(|path| {
            info!("Loading configuration from file: {}", path);
            load_config_from_file(path)
        })
        .unwrap_or_else(|| {
            warn!("No configuration file provided; using default values.");
            AppConfig {
                server: String::from(DEFAULT_SERVER),
                interval_secs: DEFAULT_INTERVAL_SECS,
            }
        });

    // Step 3: Override with environment variables if set
    let env_server = env::var("SENSOR_SERVER").unwrap_or_else(|_| file_config.server.clone());
    if env::var("SENSOR_SERVER").is_ok() {
        info!("Server address overridden by environment variable.");
    }

    let env_interval = env::var("SENSOR_INTERVAL")
        .ok()
        .and_then(|val| val.parse::<u64>().ok())
        .unwrap_or(file_config.interval_secs);
    if env::var("SENSOR_INTERVAL").is_ok() {
        info!("Interval overridden by environment variable.");
    }

    // Step 4: Override with command-line arguments if provided
    let server = matches
        .get_one::<String>("server")
        .unwrap_or(&env_server)
        .to_string();
    let interval_secs = matches
        .get_one::<u64>("interval")
        .copied()
        .unwrap_or(env_interval);

    info!(
        "Final configuration: server = {}, interval_secs = {}",
        server, interval_secs
    );

    AppConfig {
        server,
        interval_secs,
    }
}

/// Load configuration from a TOML file.
fn load_config_from_file(path: &str) -> AppConfig {
    let contents = fs::read_to_string(path).unwrap_or_else(|e| {
        error!("Failed to read configuration file at {}: {}", path, e);
        panic!("Failed to read configuration file.");
    });

    toml::from_str(&contents).unwrap_or_else(|e| {
        error!("Failed to parse configuration file at {}: {}", path, e);
        panic!("Failed to parse configuration file.");
    })
}
