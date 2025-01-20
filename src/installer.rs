use std::io;
use std::process::{Command, Stdio};

#[cfg(unix)]
use libc::geteuid;

/// Ensures the `lm-sensors` package is installed and checks for sudo access if required.
pub fn ensure_sensors_installed() -> io::Result<()> {
    if !is_command_available("sensors")? {
        eprintln!("`sensors` command not found. Attempting to install...");

        if !is_running_as_root() && !has_sudo_access()? {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Sudo privileges are required to install `lm-sensors`. Please run with sudo or contact your system administrator.",
            ));
        }

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

/// Installs the `lm-sensors` package using `apt-get`. Avoids using `sudo` if already running as root.
fn install_lm_sensors() -> io::Result<bool> {
    let mut command = Command::new("apt-get");
    if !is_running_as_root() {
        command.arg("sudo");
    }
    let status = command
        .arg("install")
        .arg("-y")
        .arg("lm-sensors")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    Ok(status.success())
}

/// Checks if the user has sudo access.
fn has_sudo_access() -> io::Result<bool> {
    let status = Command::new("sudo")
        .arg("-n") // Do not prompt for a password
        .arg("true")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match status {
        Ok(status) => Ok(status.success()),
        Err(e) => {
            eprintln!("Failed to check sudo access: {}", e);
            Ok(false)
        }
    }
}

/// Checks if the program is running as root (Unix-specific).
#[cfg(unix)]
fn is_running_as_root() -> bool {
    unsafe { geteuid() == 0 }
}

/// Only exists for local dev.
#[cfg(windows)]
fn is_running_as_root() -> bool {
    return true;
}
