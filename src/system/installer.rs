#![cfg(unix)]

//! Installer Logic
//!
//! This module ensures that required system tools (e.g., `lm-sensors`) are installed and available.

use libc::geteuid;
use log::{error, info, warn};
use crate::system::execution_util::ExecutionUtil;

/// A utility class for ensuring system tools are installed (Unix-specific).
pub struct InstallerUtil;

impl InstallerUtil {
    /// Ensures the `lm-sensors` package is installed and checks for sudo access if required.
    pub fn ensure_sensors_installed() -> bool {
        if !Self::is_command_available("sensors") {
            info!("`sensors` command not found. Attempting to install...");

            if !Self::is_running_as_root() && !Self::has_sudo_access() {
                warn!(
                    "Sudo privileges are required to install `lm-sensors`. Please run with sudo or contact your system administrator."
                );
                return false;
            }

            if Self::install_lm_sensors() {
                info!("`lm-sensors` successfully installed.");
                true
            } else {
                error!("`lm-sensors` installation failed.");
                false
            }
        } else {
            info!("`sensors` command is already installed.");
            true
        }
    }

    /// Installs the `lm-sensors` package using `apt-get`. Avoids using `sudo` if already running as root.
    fn install_lm_sensors() -> bool {
        let command = if Self::is_running_as_root() {
            "apt-get"
        } else {
            "sudo"
        };

        let args = if Self::is_running_as_root() {
            vec!["install", "-y", "lm-sensors"]
        } else {
            vec!["apt-get", "install", "-y", "lm-sensors"]
        };

        match ExecutionUtil::execute_with_command(command, &args) {
            Ok(_) => true,
            Err(e) => {
                error!("Failed to execute installation command: {}", e);
                false
            }
        }
    }

    /// Checks if the user has sudo access.
    fn has_sudo_access() -> bool {
        match ExecutionUtil::execute_with_command("sudo", &["-n", "true"]) {
            Ok(_) => true,
            Err(e) => {
                error!("Failed to check sudo access: {}", e);
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
        match ExecutionUtil::execute_with_command("which", &[command]) {
            Ok(_) => true,
            Err(e) => {
                error!("Failed to check if command `{}` is available: {}", command, e);
                false
            }
        }
    }
}
