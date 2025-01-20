mod config;
mod network;
mod sensor;

use log::error;
use log::info;
use std::thread;
use std::time::Duration;

fn main() {
    env_logger::init(); // Initialize the logger
    info!("Starting the Gilded-Sentinel-Debian application...");
    if let Err(e) = sensor::ensure_sensors_installed() {
        error!("Error ensuring lm-sensors package is installed: {}", e);
    }

    let config = config::load_config();

    info!(
        "Application running with configuration: server = {}, interval_secs = {}",
        config.server, config.interval_secs
    );

    // Main loop for data collection and transmission
    loop {
        let sensor_data = sensor::collect_sensor_data();

        match network::send_data_to_server(&sensor_data, &config.server) {
            Ok(_) => println!("Sensor data sent successfully."),
            Err(e) => eprintln!("Error sending data over socket: {}", e),
        }

        thread::sleep(Duration::from_secs(config.interval_secs));
    }
}
