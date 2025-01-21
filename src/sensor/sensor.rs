use crate::data::models::{CpuCoreData, CpuPackageData, SensorData};
use std::io;
use std::process::{Command, Stdio};

use super::mock;

/// Collects sensor data.
pub fn collect_sensor_data() -> Option<SensorData> {
    if cfg!(target_os = "windows") {        
        // Mock sensor data retrieval and parsing for Windows.
        let mock_data = mock::get_mock_sensor_data();
        Some(parse_sensor_data(&mock_data))
    } else {
        match execute_sensors_command() {
            Ok(data) => Some(parse_sensor_data(&data)),
            Err(e) => {
                eprintln!("Error retrieving sensor data: {}", e);
                None
            }
        }
    }
}

/// Executes the `sensors` command.
fn execute_sensors_command() -> io::Result<String> {
    let output = Command::new("sensors")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("sensors command failed: {}", err_msg),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Parses the output from `sensors` into structured data.
fn parse_sensor_data(raw_data: &str) -> SensorData {
    let mut cpu_packages = Vec::new();
    let mut current_package: Option<CpuPackageData> = None;

    for line in raw_data.lines() {
        if is_adapter_line(line) {
            if let Some(package) = current_package.take() {
                cpu_packages.push(package);
            }
            current_package = Some(parse_adapter_line(line));
        } else if is_package_line(line) {
            if let Some(ref mut package) = current_package {
                parse_package_line(line, package);
            }
        } else if is_core_line(line) {
            if let Some(ref mut package) = current_package {
                parse_core_line(line, package);
            }
        }
    }

    if let Some(package) = current_package {
        cpu_packages.push(package);
    }

    SensorData { cpu_packages }
}

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
