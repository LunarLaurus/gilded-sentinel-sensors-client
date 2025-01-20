mod config;
mod installer;
mod models;
mod network;
mod sensor;

use config::ConfigLoader;
use log::{error, info};
use signal_hook_registry::register;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time::Duration};

const SIGINT: i32 = libc::SIGINT;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    initialize_logger();
    info!("Starting the Gilded-Sentinel-Debian application...");

    let running = setup_signal_handler()?;
    ensure_lm_sensors_installed()?;

    let config = load_application_config();
    info!(
        "Application running with configuration: server = {}, interval_secs = {}",
        config.server, config.interval_secs
    );

    run_main_loop(&running, &config);
    info!("Shutting down gracefully.");
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

    unsafe {
        register(SIGINT, move || {
            r.store(false, Ordering::Relaxed);
        })?;
    }

    Ok(running)
}

/// Ensures that the `lm-sensors` package is installed.
fn ensure_lm_sensors_installed() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(e) = installer::ensure_sensors_installed() {
        error!("Error ensuring lm-sensors package is installed: {}", e);
        return Err(Box::new(e));
    }
    Ok(())
}

/// Loads the application configuration.
fn load_application_config() -> config::AppConfig {
    ConfigLoader::new().load_config()
}

/// Runs the main loop for data collection and transmission.
fn run_main_loop(running: &Arc<AtomicBool>, config: &config::AppConfig) {
    info!("Entering the main loop.");
    while running.load(Ordering::Relaxed) {
        process_sensor_data(&config.server);
        thread::sleep(Duration::from_secs(config.interval_secs));
    }
    info!("Exiting the main loop.");
}

/// Collects sensor data and sends it to the server.
fn process_sensor_data(server: &str) {
    match sensor::collect_sensor_data() {
        Some(sensor_data) => {
            let mut retries = 3;
            while retries > 0 {
                match network::send_data_to_server(&sensor_data, server) {
                    Ok(_) => {
                        info!("Sensor data sent successfully.");
                        return;
                    }
                    Err(e) => {
                        error!(
                            "Error sending data to server: {}. Retries left: {}",
                            e,
                            retries - 1
                        );
                        retries -= 1;
                        thread::sleep(Duration::from_secs(2));
                    }
                }
            }
            error!("Failed to send sensor data after multiple retries.");
        }
        None => error!("Failed to collect sensor data."),
    }
}
