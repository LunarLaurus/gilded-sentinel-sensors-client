use log::{debug, error, info};
use serde::Serialize;
use std::io::{self, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::thread;
use std::time::Duration;

/// A utility class for handling network operations, such as sending data to a server.
pub struct NetworkUtil;

impl NetworkUtil {
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
    ///
    /// # Logging
    /// - Logs each retry attempt and failure.
    pub fn send_with_retries<T: Serialize>(
        data: &T,
        server: &str,
        retries: usize,
    ) -> io::Result<()> {
        for attempt in 1..=retries {
            match Self::send_object_to_server(data, server) {
                Ok(_) => {
                    info!("Data successfully sent to the server on attempt {}/{}", attempt, retries);
                    return Ok(());
                }
                Err(e) => {
                    error!(
                        "Attempt {}/{}: Failed to send data to server: {}",
                        attempt, retries, e
                    );
                    if attempt < retries {
                        debug!("Retrying in 2 seconds...");
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
    ///
    /// # Parameters
    /// - `data`: The data to send, serialized as JSON.
    /// - `server`: The server address (e.g., "127.0.0.1:5000").
    ///
    /// # Returns
    /// - `Ok(())` if the data is successfully sent.
    /// - `Err(io::Error)` if the connection or transmission fails.
    ///
    /// # Logging
    /// - Logs connection success and failure, as well as serialization errors.
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

                // Send the serialized JSON data to the server.
                stream.write_all(json_data.as_bytes())?;
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
