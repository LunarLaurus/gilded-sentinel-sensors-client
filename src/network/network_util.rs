#![cfg(unix)]

use get_if_addrs::{get_if_addrs, IfAddr};
use log::{debug, error, info};
use serde::Serialize;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;
use std::{io, thread};

/// A utility class for handling network operations, such as sending data to a server.
pub struct NetworkUtil;

#[allow(dead_code)]
impl NetworkUtil {
    /// Retrieves the system's primary IP address (IPv4 or IPv6).
    pub fn get_primary() -> String {
        if let Ok(interfaces) = get_if_addrs() {
            for interface in interfaces {
                match interface.addr {
                    IfAddr::V4(v4addr) if !v4addr.ip.is_loopback() => {
                        return v4addr.ip.to_string();
                    }
                    IfAddr::V6(v6addr) if !v6addr.ip.is_loopback() => {
                        return v6addr.ip.to_string();
                    }
                    _ => {}
                }
            }
        }
        "<unknown>".to_string() // Return "<unknown>" if no valid address is found
    }

    /// Retrieves the system's primary IPv4 address.
    pub fn get_primary_ipv4() -> String {
        if let Ok(interfaces) = get_if_addrs() {
            for interface in interfaces {
                if let IfAddr::V4(v4addr) = interface.addr {
                    if !v4addr.ip.is_loopback() {
                        return v4addr.ip.to_string();
                    }
                }
            }
        }
        "<unknown>".to_string() // Return "<unknown>" if no valid address is found
    }

    /// Retrieves the system's primary IPv6 address.
    pub fn get_primary_ipv6() -> String {
        if let Ok(interfaces) = get_if_addrs() {
            for interface in interfaces {
                if let IfAddr::V6(v6addr) = interface.addr {
                    if !v6addr.ip.is_loopback() {
                        return v6addr.ip.to_string();
                    }
                }
            }
        }
        "<unknown>".to_string() // Return "<unknown>" if no valid address is found
    }

    /// Sends a generic serializable object to the server with a configurable number of retries.
    ///
    /// # Parameters
    /// - `data`: The data to send, which must implement the `Serialize` trait.
    /// - `server`: The server address (e.g., "127.0.0.1:5000").
    /// - `retries`: The maximum number of retries for sending the data.
    ///
    /// # Returns
    /// - `Ok(())` if the data is successfully sent.
    /// - `Err(io::Error)` if all retries fail.
    pub fn send_with_retries<T: Serialize>(
        data: &T,
        server: &str,
        retries: usize,
    ) -> io::Result<()> {
        Self::send_with_retries_define_timeout(
            data,
            server,
            retries,
            Duration::from_secs(2),
        )
    }

    /// Sends a generic serializable object to the server with a configurable number of retries.
    ///
    /// # Parameters
    /// - `data`: The data to send, which must implement the `Serialize` trait.
    /// - `server`: The server address (e.g., "127.0.0.1:5000").
    /// - `retries`: The maximum number of retries for sending the data.
    /// - `retry_delay`: The delay between retries.
    ///
    /// # Returns
    /// - `Ok(())` if the data is successfully sent.
    /// - `Err(io::Error)` if all retries fail.
    pub fn send_with_retries_define_timeout<T: Serialize>(
        data: &T,
        server: &str,
        retries: usize,
        retry_delay: Duration,
    ) -> io::Result<()> {
        for attempt in 1..=retries {
            match Self::send_object_to_server(data, server) {
                Ok(_) => {
                    info!(
                        "Data successfully sent to the server on attempt {}/{}",
                        attempt, retries
                    );
                    return Ok(());
                }
                Err(e) => {
                    error!(
                        "Attempt {}/{}: Failed to send data to server: {}",
                        attempt, retries, e
                    );
                    if attempt < retries {
                        debug!("Retrying in {:?}...", retry_delay);
                        thread::sleep(retry_delay);
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
    ///
    /// # Parameters
    /// - `data`: The data to send, serialized as JSON.
    /// - `server`: The server address (e.g., "127.0.0.1:5000").
    ///
    /// # Returns
    /// - `Ok(())` if the data is successfully sent.
    /// - `Err(io::Error)` if the connection or transmission fails.
    pub fn send_object_to_server<T: Serialize>(data: &T, server: &str) -> io::Result<()> {
        // Resolve the server address.
        let server_addr = server
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid server address"))?;

        info!("Connecting to server at: {}", server_addr);

        // Attempt to connect to the server with a timeout.
        let stream_result = TcpStream::connect_timeout(&server_addr, Duration::from_secs(10));

        match stream_result {
            Ok(mut stream) => {
                info!("Successfully connected to the server at {}", server_addr);

                // Serialize the data into JSON format.
                let json_data = serde_json::to_string(data).map_err(|e| {
                    error!("Serialization error: {}", e);
                    io::Error::new(io::ErrorKind::InvalidData, "Failed to serialize data")
                })?;

                debug!("Serialized data: {}", json_data);

                // Construct the HTTP request
                let host = server.split(':').next().unwrap_or("127.0.0.1");
                let request = format!(
                    "POST / HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                    host,
                    json_data.len(),
                    json_data
                );

                debug!("Constructed HTTP request: {}", request);

                // Send the HTTP request
                io::Write::write_all(&mut stream, request.as_bytes())?;
                io::Write::flush(&mut stream)?;

                info!("Data successfully sent to the server.");
                Ok(())
            }
            Err(e) => {
                error!("Failed to connect to server at {}: {}", server_addr, e);
                Err(e)
            }
        }
    }
}
