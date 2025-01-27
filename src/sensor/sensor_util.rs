#![cfg(unix)]

use log::{error, info};
use serde::Serialize;
use std::io;
use std::process::{Command, Stdio};

use crate::data::models::{CpuCoreData, CpuPackageData, SensorData, SystemInfo};
use crate::hardware::system_information_monitor::SysInfoMonitor;
use crate::network::network_util::NetworkUtil;

/// Static utility class for sensor-related operations.
///
/// Provides methods for retrieving and processing sensor data, including execution
/// of the `sensors` command, parsing the output, and sending data to the server.
#[allow(dead_code)]
pub struct SensorUtils;

impl SensorUtils {
    /// Collects CPU package data.
    ///
    /// On Unix-like systems, this executes the `sensors` command and parses its output.
    pub fn collect_cpu_package_data() -> Vec<CpuPackageData> {
        // Execute `sensors` command on Unix-like systems.
        match Self::execute_sensors_command() {
            Ok(data) => Self::parse_sensor_data(&data),
            Err(e) => {
                error!("Error retrieving sensor data: {}", e);
                Vec::new() // Return an empty vector on failure.
            }
        }
    }

    /// Executes the `sensors` command to retrieve sensor data.
    ///
    /// Captures both `stdout` and `stderr` and logs errors if the command fails.
    ///
    /// Returns the `stdout` content as a `String` on success, or logs and returns an error on failure.
    fn execute_sensors_command() -> io::Result<String> {
        let output = Command::new("sensors")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !output.status.success() {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("`sensors` command failed: {}", err_msg),
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Parses raw `sensors` command output into structured `CpuPackageData`.
    ///
    /// Lines are parsed to identify adapter, package, and core information,
    /// which are stored in a vector of `CpuPackageData`.
    fn parse_sensor_data(raw_data: &str) -> Vec<CpuPackageData> {
        let mut cpu_packages = Vec::new();
        let mut current_package: Option<CpuPackageData> = None;

        for line in raw_data.lines() {
            if Self::is_adapter_line(line) {
                if let Some(package) = current_package.take() {
                    cpu_packages.push(package);
                }
                current_package = Some(Self::parse_adapter_line(line));
            } else if Self::is_package_line(line) {
                if let Some(ref mut package) = current_package {
                    Self::parse_package_line(line, package);
                }
            } else if Self::is_core_line(line) {
                if let Some(ref mut package) = current_package {
                    Self::parse_core_line(line, package);
                }
            }
        }

        if let Some(package) = current_package {
            cpu_packages.push(package);
        }

        cpu_packages
    }

    /// Sends sensor data to the server using the `NetworkUtil`.
    pub fn process_sensor_data(server: &str, monitor: &mut SysInfoMonitor) {
        /// Sends data with retries and logs the outcome.
        fn send_and_log<T: Serialize>(data: &T, description: &str, server: &str) {
            match NetworkUtil::send_with_retries(data, server, 3) {
                Ok(_) => info!("{} data sent successfully.", description),
                Err(e) => error!("Failed to send {} data: {}.", description, e),
            }
        }

        // Collect data from the system monitor
        let cpu_info = monitor.get_cpu_info();
        let memory_info = monitor.get_memory_info();
        let disks = monitor.get_disk_info();
        let networks = monitor.get_network_info();
        let uptime = monitor.get_uptime();
        //let components = monitor.get_components_info();
        let components = Vec::new();
        let cpu_packages = Self::collect_cpu_package_data();
        let system_info: SystemInfo = SystemInfo {
            hostname: monitor.get_host_name(),
            uptime,
            management_ip: NetworkUtil::get_primary_ipv4(),
        };

        // Construct the SensorData DTO
        let sensor_data = SensorData {
            system_info,
            cpu_info,
            memory_info,
            disks,
            network_interfaces: networks,
            components,
            cpu_packages,
        };

        // Send data to the server
        send_and_log(&sensor_data, "SensorDataDTO", server);
    }

    // --------------------------------------
    // Line Identification Functions
    // --------------------------------------

    /// Checks if a line indicates an adapter.
    fn is_adapter_line(line: &str) -> bool {
        line.contains("coretemp-")
    }

    /// Checks if a line indicates a package.
    fn is_package_line(line: &str) -> bool {
        line.contains("Package id")
    }

    /// Checks if a line indicates a core.
    fn is_core_line(line: &str) -> bool {
        line.contains("Core")
    }

    // --------------------------------------
    // Parsing Functions
    // --------------------------------------

    /// Parses an adapter line into a `CpuPackageData` placeholder.
    fn parse_adapter_line(line: &str) -> CpuPackageData {
        let adapter_name = line
            .split_whitespace()
            .next()
            .unwrap_or("Unknown")
            .to_string();
        CpuPackageData {
            package_id: String::new(),
            adapter_name,
            package_temperature: 0.0,
            high_threshold: 0.0,
            critical_threshold: 0.0,
            cores: Vec::new(),
        }
    }

    /// Parses a package line and updates the `CpuPackageData`.
    fn parse_package_line(line: &str, package: &mut CpuPackageData) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 10 {
            package.package_id = parts[2].to_string();
            package.package_temperature = parts[3]
                .trim_start_matches('+')
                .trim_end_matches("°C")
                .parse()
                .unwrap_or(0.0);
            package.high_threshold = parts[6]
                .trim_start_matches('+')
                .trim_end_matches("°C")
                .parse()
                .unwrap_or(0.0);
            package.critical_threshold = parts[9]
                .trim_start_matches('+')
                .trim_end_matches("°C")
                .parse()
                .unwrap_or(0.0);
        }
    }

    /// Parses a core line and adds a `CpuCoreData` to the `CpuPackageData`.
    fn parse_core_line(line: &str, package: &mut CpuPackageData) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 {
            let core_data = CpuCoreData {
                core_name: parts[0].to_string(),
                temperature: parts[1]
                    .trim_start_matches('+')
                    .trim_end_matches("°C")
                    .parse()
                    .unwrap_or(0.0),
                high_threshold: parts[4]
                    .trim_start_matches('+')
                    .trim_end_matches("°C")
                    .parse()
                    .unwrap_or(0.0),
                critical_threshold: parts[5]
                    .trim_start_matches('+')
                    .trim_end_matches("°C")
                    .parse()
                    .unwrap_or(0.0),
            };
            package.cores.push(core_data);
        }
    }
}
