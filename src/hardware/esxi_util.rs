use log::{self, debug, error, info, warn};
use std::fs::OpenOptions;
use std::os::fd::AsRawFd;
use std::process::{Command, Stdio};
use std::string::FromUtf8Error;

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

        // Construct the shell command string
        let command_str = format!("{} {}", command, args.join(" "));
        debug!("Executing shell command: {}", command_str);

        // Spawn the command using `sh -c` to avoid TTY assumptions
        let output_result = Command::new("sh")
            .arg("-c")
            .arg(&command_str)
            .stdin(Stdio::null()) // Prevent TTY input
            .stdout(Stdio::piped()) // Capture standard output
            .stderr(Stdio::piped()) // Capture standard error
            .output();

        // Analyze the output
        match output_result {
            Ok(output) => {
                if output.status.success() {
                    let stdout = Self::convert_to_string(output.stdout);
                    debug!(
                        "Command `{}` succeeded with output: {}",
                        command_str, stdout
                    );
                    Ok(stdout)
                } else {
                    let stderr = Self::convert_to_string(output.stderr);
                    error!(
                        "Command `{}` failed with exit code {:?}: {}",
                        command_str,
                        output.status.code(),
                        stderr
                    );
                    Err(stderr)
                }
            }
            Err(e) => {
                error!(
                    "Failed to execute command `{}` due to an error: {}",
                    command_str, e
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
    
}
