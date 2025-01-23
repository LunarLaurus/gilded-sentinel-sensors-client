//! Installer Logic
//!
//! This module ensures that required system tools (e.g., `lm-sensors`) are installed and available.

#[cfg(unix)]
mod unix {
    use libc::geteuid;
    use std::process::{Command, Stdio};

    /// Ensures the `lm-sensors` package is installed and checks for sudo access if required.
    pub fn ensure_sensors_installed() -> bool {
        if !is_command_available("sensors") {
            eprintln!("`sensors` command not found. Attempting to install...");

            if !is_running_as_root() && !has_sudo_access() {
                eprintln!(
                    "Sudo privileges are required to install `lm-sensors`. Please run with sudo or contact your system administrator."
                );
                return false;
            }

            if install_lm_sensors() {
                eprintln!("`lm-sensors` successfully installed.");
                return true;
            } else {
                eprintln!("`lm-sensors` installation failed.");
                return false;
            }
        } else {
            eprintln!("`sensors` command is already installed.");
            true
        }
    }

    /// Installs the `lm-sensors` package using `apt-get`. Avoids using `sudo` if already running as root.
    fn install_lm_sensors() -> bool {
        let mut command = if is_running_as_root() {
            Command::new("apt-get")
        } else {
            let mut cmd = Command::new("sudo");
            cmd.arg("apt-get");
            cmd
        };

        let status = command
            .arg("install")
            .arg("-y")
            .arg("lm-sensors")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status();

        match status {
            Ok(status) => status.success(),
            Err(e) => {
                eprintln!("Failed to execute installation command: {}", e);
                false
            }
        }
    }

    /// Checks if the user has sudo access.
    fn has_sudo_access() -> bool {
        let status = Command::new("sudo")
            .arg("-n") // Do not prompt for a password
            .arg("true")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        match status {
            Ok(status) => status.success(),
            Err(e) => {
                eprintln!("Failed to check sudo access: {}", e);
                false
            }
        }
    }

    /// Checks if the program is running as root (Unix-specific).
    fn is_running_as_root() -> bool {
        unsafe { geteuid() == 0 }
    }

    /// Checks if a command is available in the system.
    fn is_command_available(command: &str) -> bool {
        let status = Command::new("which")
            .arg(command)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        match status {
            Ok(status) => status.success(),
            Err(e) => {
                eprintln!(
                    "Failed to check if command `{}` is available: {}",
                    command, e
                );
                false
            }
        }
    }
}

#[cfg(windows)]
mod windows {
    use super::*;

    /// Mocked implementation for `ensure_sensors_installed` for Windows development builds.
    pub fn ensure_sensors_installed() -> bool {
        // No-op for Windows, always returns true.
        true
    }
}

/// Re-exports platform-specific `ensure_sensors_installed` function.
#[cfg(unix)]
pub use unix::ensure_sensors_installed;

#[cfg(windows)]
pub use windows::ensure_sensors_installed;
