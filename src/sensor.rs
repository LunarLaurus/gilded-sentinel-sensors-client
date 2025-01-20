use std::io;
use std::process::{Command, Stdio};

mod mock;

pub fn collect_sensor_data() -> String {
    if cfg!(target_os = "windows") {
        mock::get_mock_sensor_data()
    } else {
        match execute_sensors_command() {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Error retrieving sensor data: {}", e);
                String::new()
            }
        }
    }
}

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

/// Checks if the `sensors` package is installed, and installs it if not.
pub fn ensure_sensors_installed() -> io::Result<()> {
    let check_output = Command::new("which")
        .arg("sensors")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if !check_output.status.success() {
        eprintln!("`sensors` command not found. Attempting to install...");

        let install_output = Command::new("sudo")
            .arg("apt-get")
            .arg("install")
            .arg("-y")
            .arg("lm-sensors")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()?;

        if !install_output.status.success() {
            let err_msg = String::from_utf8_lossy(&install_output.stderr);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to install `lm-sensors`: {}", err_msg),
            ));
        }

        eprintln!("`lm-sensors` successfully installed.");
    } else {
        eprintln!("`sensors` command is already installed.");
    }

    Ok(())
}
