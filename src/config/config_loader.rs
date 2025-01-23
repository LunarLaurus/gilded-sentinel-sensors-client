use clap::{Arg, Command};
use log::{debug, error, info, warn};
use std::env;
use std::fs;
use std::path::Path;

/// Application configuration structure.
///
/// This structure holds configuration values for the Gilded-Sentinel application,
/// such as the server address, data collection interval, and execution method.
#[derive(Debug, serde::Deserialize)]
pub struct AppConfig {
    /// Server address to which the application will send data (e.g., `127.0.0.1:5000`).
    pub server: String,
    /// Interval in seconds between data collection.
    pub interval_secs: u64,
    /// Command execution method (e.g., "std_command", "execv").
    pub execution_method: String,
}

impl Default for AppConfig {
    /// Provides default values for the application configuration.
    fn default() -> Self {
        Self {
            server: "127.0.0.1:5000".to_string(),
            interval_secs: 10,
            execution_method: "std_command".to_string(),
        }
    }
}

/// Configuration loader for the Gilded-Sentinel application.
///
/// This loader retrieves configuration values from multiple sources, such as:
/// - Configuration files (`config.toml`)
/// - Environment variables
/// - Command-line arguments
pub struct ConfigLoader {
    exe_dir: String,
}

impl ConfigLoader {
    /// Creates a new `ConfigLoader` instance with the executable's directory.
    ///
    /// This ensures that configuration files can be loaded relative to the executable's location.
    pub fn new() -> Self {
        let exe_dir = env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(|p| p.to_string_lossy().to_string()))
            .unwrap_or_else(|| ".".to_string());

        Self { exe_dir }
    }

    /// Loads the complete application configuration by combining:
    /// 1. Configuration file (`config.toml`).
    /// 2. Environment variables.
    /// 3. Command-line arguments.
    ///
    /// Returns the final `AppConfig`.
    pub fn load_config(&self) -> AppConfig {
        info!("Starting configuration loading process.");

        // Step 1: Load configuration from file
        let file_config = self.load_from_file().unwrap_or_else(|| {
            warn!("No configuration file found; using default values.");
            AppConfig::default()
        });

        // Step 2: Override with environment variables
        let env_config = self.override_with_env(file_config);

        // Step 3: Override with command-line arguments
        let final_config = self.override_with_cli(env_config);

        info!(
            "Final configuration: server = {}, interval_secs = {}, execution_method = {}",
            final_config.server, final_config.interval_secs, final_config.execution_method
        );

        final_config
    }

    /// Loads configuration from the `config.toml` file in the executable's directory.
    ///
    /// If the file is not found or cannot be parsed, this function logs the error
    /// and returns `None`.
    fn load_from_file(&self) -> Option<AppConfig> {
        let config_path = Path::new(&self.exe_dir).join("config.toml");

        if config_path.exists() {
            info!("Found configuration file at: {}", config_path.display());
            match fs::read_to_string(&config_path) {
                Ok(contents) => match toml::from_str(&contents) {
                    Ok(config) => Some(config),
                    Err(e) => {
                        error!("Failed to parse configuration file: {}", e);
                        None
                    }
                },
                Err(e) => {
                    error!("Failed to read configuration file: {}", e);
                    None
                }
            }
        } else {
            warn!("No configuration file found in: {}", self.exe_dir);
            None
        }
    }

    /// Overrides the provided configuration with values from environment variables.
    ///
    /// Supported environment variables:
    /// - `SENSOR_SERVER`: Overrides the `server` value.
    /// - `SENSOR_INTERVAL`: Overrides the `interval_secs` value.
    /// - `SENSOR_EXECUTION_METHOD`: Overrides the `execution_method` value.
    ///
    /// Logs any overridden values for traceability.
    fn override_with_env(&self, config: AppConfig) -> AppConfig {
        let server = env::var("SENSOR_SERVER").unwrap_or_else(|_| config.server.clone());
        let interval_secs = env::var("SENSOR_INTERVAL")
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(config.interval_secs);
        let execution_method =
            env::var("SENSOR_EXECUTION_METHOD").unwrap_or_else(|_| config.execution_method.clone());

        if server != config.server {
            info!("Server address overridden by environment variable.");
        }
        if interval_secs != config.interval_secs {
            info!("Interval overridden by environment variable.");
        }
        if execution_method != config.execution_method {
            info!("Execution method overridden by environment variable.");
        }

        AppConfig {
            server,
            interval_secs,
            execution_method,
        }
    }

    /// Overrides the provided configuration with values from command-line arguments.
    ///
    /// Supported arguments:
    /// - `--server`: Overrides the `server` value.
    /// - `--interval`: Overrides the `interval_secs` value.
    /// - `--execution-method`: Overrides the `execution_method` value.
    ///
    /// Logs any overridden values for traceability.
    fn override_with_cli(&self, config: AppConfig) -> AppConfig {
        let matches = Command::new("Gilded-Sentinel-Client")
            .arg(
                Arg::new("server")
                    .long("server")
                    .help("Server address to send data (e.g., 127.0.0.1:5000)")
                    .value_parser(clap::value_parser!(String)),
            )
            .arg(
                Arg::new("interval")
                    .long("interval")
                    .help("Interval in seconds between data collection")
                    .value_parser(clap::value_parser!(u64)),
            )
            .arg(
                Arg::new("execution-method")
                    .long("execution-method")
                    .help("Command execution method: [std_command (default), no_fork, execv, libc, direct_check]")
                    .value_parser(clap::value_parser!(String)),
            )
            .get_matches();

        debug!("Command-line arguments parsed successfully.");

        let server = matches
            .get_one::<String>("server")
            .unwrap_or(&config.server)
            .to_string();

        let interval_secs = matches
            .get_one::<u64>("interval")
            .copied()
            .unwrap_or(config.interval_secs);

        let execution_method = matches
            .get_one::<String>("execution-method")
            .unwrap_or(&config.execution_method)
            .to_string();

        if server != config.server {
            info!("Server address overridden by command-line argument.");
        }
        if interval_secs != config.interval_secs {
            info!("Interval overridden by command-line argument.");
        }
        if execution_method != config.execution_method {
            info!("Execution method overridden by command-line argument.");
        }

        AppConfig {
            server,
            interval_secs,
            execution_method,
        }
    }
}

/// Initializes the logger for the application.
///
/// This function sets up the `env_logger` backend to handle logging, allowing
/// log levels to be dynamically adjusted via environment variables (e.g., `RUST_LOG`).
pub fn initialize_logger() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
}

/// Loads the application configuration by using the `ConfigLoader`.
///
/// This function acts as a simple entry point for loading the configuration,
/// combining values from files, environment variables, and command-line arguments.
pub fn load_application_config() -> AppConfig {
    ConfigLoader::new().load_config()
}
