use clap::{App, Arg};
use serde::Deserialize;
use std::fs;
use std::env;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: String,
    pub interval_secs: u64,
}

// Default configuration constants
const DEFAULT_SERVER: &str = "127.0.0.1:5000";
const DEFAULT_INTERVAL_SECS: u64 = 5;

/// Load configuration from command-line arguments, configuration file, or environment variables.
pub fn load_config() -> AppConfig {
    // Step 1: Parse command-line arguments using `clap`
    let matches = App::new("Sensor App")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
        .about("Collects and sends sensor data")
        .arg(
            Arg::with_name("server")
                .long("server")
                .help("Server address (e.g., 127.0.0.1:5000)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("interval")
                .long("interval")
                .help("Interval in seconds between data collection")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("config")
                .long("config")
                .help("Path to the configuration file")
                .takes_value(true),
        )
        .get_matches();

    // Step 2: Load configuration from file if provided
    let file_config = matches
        .value_of("config")
        .map(|path| load_config_from_file(path))
        .unwrap_or_else(|| AppConfig {
            server: String::from(DEFAULT_SERVER),
            interval_secs: DEFAULT_INTERVAL_SECS,
        });

    // Step 3: Override with environment variables if set
    let env_server = env::var("SENSOR_SERVER").unwrap_or_else(|_| file_config.server.clone());
    let env_interval = env::var("SENSOR_INTERVAL")
        .ok()
        .and_then(|val| val.parse::<u64>().ok())
        .unwrap_or(file_config.interval_secs);

    // Step 4: Override with command-line arguments if provided
    let server = matches.value_of("server").unwrap_or(&env_server).to_string();
    let interval_secs = matches
        .value_of("interval")
        .and_then(|val| val.parse::<u64>().ok())
        .unwrap_or(env_interval);

    AppConfig { server, interval_secs }
}

/// Load configuration from a TOML file.
fn load_config_from_file(path: &str) -> AppConfig {
    let contents = fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("Failed to read configuration file at {}", path));
    toml::from_str(&contents)
        .unwrap_or_else(|_| panic!("Failed to parse configuration file at {}", path))
}
