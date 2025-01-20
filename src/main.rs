mod config;
mod sensor;
mod network;

use std::thread;
use std::time::Duration;

fn main() {
    // Load configuration
    let config = config::load_config();

    println!(
        "Reading sensor data every {} seconds and sending to {}...",
        config.interval_secs, config.server
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
