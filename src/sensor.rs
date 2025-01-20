use crate::models::{CpuPackageData, SensorData};
use std::io;
use std::process::{Command, Stdio};

/// Collects sensor data.
pub fn collect_sensor_data() -> Option<SensorData> {
    match execute_sensors_command() {
        Ok(data) => Some(parse_sensor_data(&data)),
        Err(e) => {
            eprintln!("Error retrieving sensor data: {}", e);
            None
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
    let mut other_sensors = Vec::new();
    let mut current_package: Option<CpuPackageData> = None;

    for line in raw_data.lines() {
        if line.contains("Package id") {
            if let Some(package) = current_package.take() {
                cpu_packages.push(package);
            }
            let package_id = line
                .split_whitespace()
                .nth(2)
                .unwrap_or("Unknown")
                .to_string();
            current_package = Some(CpuPackageData {
                package_id,
                cores: Vec::new(),
            });
        } else if line.contains("Core") {
            if let Some(ref mut package) = current_package {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 6 {
                    let core_data = crate::models::CpuCoreData {
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
        } else if !line.trim().is_empty() {
            other_sensors.push(line.to_string());
        }
    }

    if let Some(package) = current_package {
        cpu_packages.push(package);
    }

    SensorData {
        cpu_packages,
        other_sensors,
    }
}
