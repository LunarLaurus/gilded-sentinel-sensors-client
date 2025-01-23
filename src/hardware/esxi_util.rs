use std::process::{Command, Stdio};
use std::string::FromUtf8Error;
use log::{self, debug, error, info, warn};

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
        debug!("Attempting to execute command: `{}` with args: {:?}", command, args);

        // Spawn the command and capture output
        let output_result = Command::new(command)
            .args(args)
            .stdin(Stdio::null()) // Prevent TTY input
            .stdout(Stdio::piped()) // Capture standard output
            .stderr(Stdio::piped()) // Capture standard error
            .output();

        // Analyze the output
        match output_result {
            Ok(output) => {
                if output.status.success() {
                    let stdout = Self::convert_to_string(output.stdout);
                    debug!("Command `{}` succeeded with output: {}", command, stdout);
                    Ok(stdout)
                } else {
                    let stderr = Self::convert_to_string(output.stderr);
                    error!(
                        "Command `{}` with args {:?} failed (exit code: {:?}): {}",
                        command,
                        args,
                        output.status.code(),
                        stderr
                    );
                    Err(stderr)
                }
            }
            Err(e) => {
                error!(
                    "Failed to execute command `{}` with args {:?}: {:?}",
                    command,
                    args,
                    e
                );
                Err(e.to_string())
            }
        }
    }

    /// Helper method to safely convert `Vec<u8>` to `String` while handling potential errors.
    fn convert_to_string(output: Vec<u8>) -> String {
        match String::from_utf8(output) {
            Ok(s) => s,
            Err(FromUtf8Error { .. }) => "<Invalid UTF-8 Output>".to_string(),
        }
    }

    /// Debugging method to log environment-specific details for deeper analysis.
    pub fn log_environment_details() {
        info!("Logging environment details for debugging...");
        
        // Check if running as root
        let user_id = unsafe { libc::geteuid() };
        if user_id == 0 {
            info!("Running as root user (UID: 0)");
        } else {
            warn!("Not running as root user (UID: {})", user_id);
        }

        // Log basic system information
        if let Ok(uname_output) = Self::execute_command("uname", &["-a"]) {
            info!("System Information (uname -a): {}", uname_output.trim());
        } else {
            warn!("Failed to retrieve system information using `uname`.");
        }

        // Check if `vsish` exists manually
        if let Ok(_) = Self::execute_command("ls", &["/bin/vsish"]) {
            info!("`/bin/vsish` exists and is accessible.");
        } else {
            warn!("`/bin/vsish` is not found or inaccessible.");
        }
    }
}
