use std::io;
use std::process::{Command, Stdio};

/// Ensures the `lm-sensors` package is installed.
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
