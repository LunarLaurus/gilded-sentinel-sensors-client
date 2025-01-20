use std::io;
use std::process::{Command, Stdio};

/// Ensures the `lm-sensors` package is installed.
pub fn ensure_sensors_installed() -> io::Result<()> {
    if !is_command_available("sensors")? {
        eprintln!("`sensors` command not found. Attempting to install...");

        if install_lm_sensors()? {
            eprintln!("`lm-sensors` successfully installed.");
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "`lm-sensors` installation failed.",
            ));
        }
    } else {
        eprintln!("`sensors` command is already installed.");
    }

    Ok(())
}

/// Checks if a command is available in the system.
fn is_command_available(command: &str) -> io::Result<bool> {
    let status = Command::new("which")
        .arg(command)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    Ok(status.success())
}

/// Installs the `lm-sensors` package using `apt-get`.
fn install_lm_sensors() -> io::Result<bool> {
    let status = Command::new("sudo")
        .arg("apt-get")
        .arg("install")
        .arg("-y")
        .arg("lm-sensors")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    Ok(status.success())
}