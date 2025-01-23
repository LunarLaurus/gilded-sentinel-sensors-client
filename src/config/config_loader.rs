use clap::{Arg, Command};
use log::{debug, error, info, warn};
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, serde::Deserialize)]
pub struct AppConfig {
    pub server: String,
    pub interval_secs: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: "127.0.0.1:5000".to_string(),
            interval_secs: 10,
        }
    }
}

pub struct ConfigLoader {
    exe_dir: String,
}

pub fn initialize_logger() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
}

pub fn load_application_config() -> AppConfig {
    crate::config::config_loader::ConfigLoader::new().load_config()
}

impl ConfigLoader {
    /// Creates a new ConfigLoader with the executable's directory.
    pub fn new() -> Self {
        let exe_dir = env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(|p| p.to_string_lossy().to_string()))
            .unwrap_or_else(|| ".".to_string());

        Self { exe_dir }
    }

    /// Load the final application configuration.
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
            "Final configuration: server = {}, interval_secs = {}",
            final_config.server, final_config.interval_secs
        );

        final_config
    }

    /// Loads configuration from `config.toml` in the local directory.
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

    /// Overrides configuration with environment variables if set.
    fn override_with_env(&self, config: AppConfig) -> AppConfig {
        let server = env::var("SENSOR_SERVER").unwrap_or_else(|_| config.server.clone());
        let interval_secs = env::var("SENSOR_INTERVAL")
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(config.interval_secs);

        if server != config.server {
            info!("Server address overridden by environment variable.");
        }
        if interval_secs != config.interval_secs {
            info!("Interval overridden by environment variable.");
        }

        AppConfig {
            server,
            interval_secs,
        }
    }

    /// Overrides configuration with command-line arguments if provided.
    fn override_with_cli(&self, config: AppConfig) -> AppConfig {
        let matches = Command::new("Gilded-Sentinel-Debian")
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

        if server != config.server {
            info!("Server address overridden by command-line argument.");
        }
        if interval_secs != config.interval_secs {
            info!("Interval overridden by command-line argument.");
        }

        AppConfig {
            server,
            interval_secs,
        }
    }
    
}
