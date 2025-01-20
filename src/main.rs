mod config;
mod sensor;
mod network;

use std::thread;
use std::time::Duration;

fn main() {
    let interval_secs = config::get_interval_from_env();

    println!("Reading sensor data every {} seconds...", interval_secs);

    loop {
        let sensor_data = sensor::collect_sensor_data();

        match network::send_data_to_server(&sensor_data, config::SERVER) {
            Ok(_) => println!("Sensor data sent successfully."),
            Err(e) => eprintln!("Error sending data over socket: {}", e),
        }

        thread::sleep(Duration::from_secs(interval_secs));
    }
}
