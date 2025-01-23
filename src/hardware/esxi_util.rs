use log::{self, debug, info, warn};
use std::fs::OpenOptions;
use std::os::fd::AsRawFd;

use crate::system::execution_util::ExecutionUtil;

/// A utility class for interacting with the ESXi environment.
pub struct EsxiUtil;

impl EsxiUtil {
    // -----------------------------------
    // Environment & TTY Checking
    // -----------------------------------

    /// Checks if the program is running in a TTY environment (Unix-based).
    #[cfg(unix)]
    pub fn is_tty() -> bool {
        use std::os::unix::io::AsRawFd;
        unsafe { libc::isatty(std::io::stdout().as_raw_fd()) != 0 }
    }

    /// Placeholder for non-Unix platforms.
    #[cfg(not(unix))]
    pub fn is_tty() -> bool {
        true
    }

    pub fn redirect_to_null() {
        let dev_null = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/null")
            .unwrap();
        unsafe {
            libc::dup2(dev_null.as_raw_fd(), libc::STDIN_FILENO);
            //libc::dup2(dev_null.as_raw_fd(), libc::STDOUT_FILENO);
            //libc::dup2(dev_null.as_raw_fd(), libc::STDERR_FILENO);
        }
    }

    /// Checks if the system is running on ESXi by verifying the presence of the `vsish` command.
    pub fn is_running_on_esxi() -> bool {
        match Self::execute_command("which", &["vsish"]) {
            Ok(output) => {
                info!("`which vsish` output: {}", output.trim());
                true
            }
            Err(err) => {
                warn!("Failed to detect ESXi environment: {}", err);
                false
            }
        }
    }

/// Executes a command without a TTY, captures output, handles errors, and logs details.
pub fn execute_command(command: &str, args: &[&str]) -> Result<String, String> {
    debug!(
        "Attempting to execute command: `{}` with args: {:?}",
        command, args
    );

    // Call the utility function and handle its result
    match ExecutionUtil::execute_with_libc(command, args) {
        Ok(output) => {
            info!("Command succeeded with output: {}", output);
            Ok(output) // Return the success result
        }
        Err(error) => {
            info!("Command failed with error: {}", error);
            Err(error) // Return the error result
        }
    }
}

}
