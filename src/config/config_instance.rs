use crate::config::AppConfig;
use std::sync::OnceLock;

pub struct Config;

// Static instance of the configuration
static CONFIG_INSTANCE: OnceLock<AppConfig> = OnceLock::new();

impl Config {
    /// Initializes the global configuration. Can only be called once.
    pub fn initialize(config: AppConfig) {
        CONFIG_INSTANCE
            .set(config)
            .expect("Configuration can only be initialized once");
    }

    /// Retrieves a reference to the global configuration.
    ///
    /// # Panics
    /// Panics if the configuration has not been initialized.
    pub fn get() -> &'static AppConfig {
        CONFIG_INSTANCE
            .get()
            .expect("Configuration must be initialized")
    }

    /// Convenience method for getting the execution method.
    pub fn execution_method() -> &'static str {
        &Config::get().execution_method
    }

    /// Convenience method for getting the server address.
    pub fn server() -> &'static str {
        &Config::get().server
    }

    /// Convenience method for getting the interval.
    pub fn interval_secs() -> u64 {
        Config::get().interval_secs
    }
}
