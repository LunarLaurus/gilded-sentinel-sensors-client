mod config;
mod network;
mod sensor;

use config::ConfigLoader;
use log::{error, info};
use signal_hook::flag;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    initialize_logger();
    info!("Starting the Gilded-Sentinel-Debian application...");

    let running = setup_signal_handler()?;

    if let Err(e) = sensor::ensure_sensors_installed() {
        handle_initialization_error("Error ensuring lm-sensors package is installed", e)?;
    }

    let config = load_application_config();
    info!(
        "Application running with configuration: server = {}, interval_secs = {}",
        config.server, config.interval_secs
    );

    run_main_loop(&running, &config);

    info!("Shutting down gracefully...");
    Ok(())
}

/// Initializes the logger with default INFO level.
fn initialize_logger() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
}

/// Sets up a signal handler for graceful shutdown.
fn setup_signal_handler() -> Result<Arc<AtomicBool>, Box<dyn std::error::Error>> {
    let running = Arc::new(AtomicBool::new(true));
    let r = Arc::clone(&running);
    flag::register(signal_hook::consts::SIGINT, r)?;
    Ok(running)
}

/// Handles initialization errors, logging the message and returning an error.
fn handle_initialization_error(message: &str, error: impl std::error::Error) -> Result<(), Box<dyn std::error::Error>> {
    error!("{}: {}", message, error);
    Err(Box::new(error))
}

/// Loads the application configuration.
fn load_application_config() -> config::AppConfig {
    let config_loader = ConfigLoader::new();
    config_loader.load_config()
}

/// Runs the main loop for data collection and transmission.
fn run_main_loop(running: &Arc<AtomicBool>, config: &config::AppConfig) {
    while running.load(Ordering::Relaxed) {
        if let Some(sensor_data) = sensor::collect_sensor_data() {
            if let Err(e) = network::send_data_to_server(&sensor_data, &config.server) {
                error!("Error sending data to server: {}", e);
            } else {
                info!("Sensor data sent successfully.");
            }
        } else {
            error!("Failed to collect sensor data.");
        }

        thread::sleep(Duration::from_secs(config.interval_secs));
    }
}
