use log::{error, info};
use serde::Serialize;
use std::io::{self, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::thread;
use std::time::Duration;

use crate::data::models::SensorData;
use crate::hardware::system_information_monitor::SysInfoMonitor;
use crate::sensor::sensor_util::collect_cpu_package_data;

pub struct NetworkUtil;

impl NetworkUtil {
    /// Sends a generic serializable object to the server with retries.
    pub fn send_with_retries<T: Serialize>(
        data: &T,
        server: &str,
        retries: usize,
    ) -> io::Result<()> {
        for attempt in (1..=retries).rev() {
            match Self::send_object_to_server(data, server) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    error!(
                        "Error sending data to server: {}. Retries left: {}",
                        e,
                        attempt - 1
                    );
                    if attempt > 1 {
                        thread::sleep(Duration::from_secs(2));
                    }
                }
            }
        }
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to send data after multiple retries.",
        ))
    }

    /// Sends a generic serializable object as JSON to the server.
    pub fn send_object_to_server<T: Serialize>(data: &T, server: &str) -> io::Result<()> {
        let server_addr = server
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid server address"))?;

        info!("Connecting to server at: {}", server_addr);
        let stream_result = TcpStream::connect_timeout(&server_addr, Duration::from_secs(10));

        match stream_result {
            Ok(mut stream) => {
                info!("Successfully connected to the server at {}", server_addr);

                let json_data = serde_json::to_string(data).map_err(|e| {
                    error!("Failed to serialize data: {}", e);
                    io::Error::new(io::ErrorKind::Other, "Serialization error")
                })?;

                stream.write_all(json_data.as_bytes())?;
                info!("Successfully sent data to the server.");
                Ok(())
            }
            Err(e) => {
                error!("Failed to connect to server at {}: {}", server_addr, e);
                Err(e)
            }
        }
    }

    /// Collects and sends sensor data to the server.
    pub fn process_sensor_data(server: &str, monitor: &mut SysInfoMonitor) {
        // Define a reusable function for sending and logging
        fn send_and_log<T: Serialize>(data: &T, description: &str, server: &str) {
            match NetworkUtil::send_with_retries(data, server, 3) {
                Ok(_) => info!("{} data sent successfully.", description),
                Err(e) => error!("Failed to send {} data: {}.", description, e),
            }
        }

        // Retrieve all required data from the monitor
        let cpu_info = monitor.get_cpu_info();
        let memory_info = monitor.get_memory_info();
        let disks = monitor.get_disk_info();
        let networks = monitor.get_network_info();
        let uptime = monitor.get_uptime();
        let components = monitor.get_components_info();
        let cpu_packages = collect_cpu_package_data();

        // Construct the SensorData DTO using retrieved data
        let sensor_data = SensorData {
            uptime,
            cpu_info,
            memory_info,
            disks,
            network_interfaces: networks,
            components,
            cpu_packages,
        };

        // Send the complete SensorData DTO
        send_and_log(&sensor_data, "SensorDataDTO", server);
    }
}
