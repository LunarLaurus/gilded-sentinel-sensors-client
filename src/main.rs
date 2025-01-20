mod config;
mod network;
mod sensor;

use log::{error, info};
use signal_hook::flag;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the logger with default INFO level.
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("Starting the Gilded-Sentinel-Debian application...");

    // Handle Ctrl+C for graceful shutdown.
    let running = Arc::new(AtomicBool::new(true));
    let r = Arc::clone(&running);
    flag::register(signal_hook::consts::SIGINT, r)?;

    if let Err(e) = sensor::ensure_sensors_installed() {
        error!("Error ensuring lm-sensors package is installed: {}", e);
        return Err(e.into());
    }

    let config = config::load_config();
    info!(
        "Application running with configuration: server = {}, interval_secs = {}",
        config.server, config.interval_secs
    );

    // Main loop for data collection and transmission.
    while running.load(Ordering::Relaxed) {
        if let Some(sensor_data) = sensor::collect_sensor_data() {
            match network::send_data_to_server(&sensor_data, &config.server) {
                Ok(_) => info!("Sensor data sent successfully."),
                Err(e) => error!("Error sending data to server: {}", e),
            }
        } else {
            error!("Failed to collect sensor data.");
        }

        thread::sleep(Duration::from_secs(config.interval_secs));
    }

    info!("Shutting down gracefully...");
    Ok(())
}
