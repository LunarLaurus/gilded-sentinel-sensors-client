#![cfg(unix)]

use crate::{config::config::Config, system::execution_util::ExecutionUtil};
use log::{self, debug, info};

/// A utility class for interacting with the ESXi environment.
pub struct EsxiUtil;

// Unix-specific implementations
impl EsxiUtil {
    /// Executes a command without a TTY, captures output, handles errors, and logs details.
    pub fn execute_command(command: &str, args: &[&str]) -> Result<String, String> {
        debug!(
            "Attempting to execute command: `{}` with args: {:?}",
            command, args
        );

        // Call the utility function and handle its result
        match ExecutionUtil::execute(Config::execution_method(), command, args) {
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
