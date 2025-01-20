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
