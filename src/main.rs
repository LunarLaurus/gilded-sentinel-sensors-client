mod config;
mod sensor;
mod network;

use std::thread;
use std::time::Duration;
use log::info;

fn main() {
    
    env_logger::init(); // Initialize the logger
    info!("Starting the Gilded-Sentinel-Debian application...");

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
