use crate::models::SensorData;
use serde_json;
use std::io::{self, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;
use log::{error, info};

/// Sends sensor data to the server.
pub fn send_data_to_server(data: &SensorData, server: &str) -> io::Result<()> {
    // Convert server address to a socket address with timeout
    let server_addr = server.to_socket_addrs()?.next().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "Invalid server address")
    })?;

    info!("Connecting to server at: {}", server_addr);
    let stream_result = TcpStream::connect_timeout(&server_addr, Duration::from_secs(10));

    match stream_result {
        Ok(mut stream) => {
            info!("Successfully connected to the server at {}", server_addr);

            let json_data = match serde_json::to_string(data) {
                Ok(json) => json,
                Err(e) => {
                    error!("Failed to serialize sensor data: {}", e);
                    return Err(io::Error::new(io::ErrorKind::Other, "Serialization error"));
                }
            };

            stream.write_all(json_data.as_bytes())?;
            info!("Successfully sent sensor data to the server.");
            Ok(())
        }
        Err(e) => {
            error!("Failed to connect to server at {}: {}", server_addr, e);
            Err(e)
        }
    }
}
