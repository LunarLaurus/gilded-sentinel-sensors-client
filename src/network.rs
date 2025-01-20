use crate::sensor::SensorData;
use serde_json;
use std::io::{self, Write};
use std::net::TcpStream;

/// Sends sensor data to the server.
pub fn send_data_to_server(data: &SensorData, server: &str) -> io::Result<()> {
    let mut stream = TcpStream::connect(server)?;
    let json_data = serde_json::to_string(data).expect("Failed to serialize sensor data");
    stream.write_all(json_data.as_bytes())?;
    Ok(())
}
