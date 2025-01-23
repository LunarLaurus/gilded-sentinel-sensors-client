#![cfg(unix)]

use log::{debug, error};
use nix::sys::signal::Signal;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{execv, fork, ForkResult};
use std::ffi::CString;
use std::process::{Command, Stdio};

/// Utility class for executing commands in various ways.
pub struct ExecutionUtil;

#[allow(dead_code)]
impl ExecutionUtil {
    /// Dispatches command execution based on the method specified.
    ///
    /// # Arguments
    /// - `method`: The method to execute the command (e.g., "no_fork", "execv", "std_command", "libc").
    /// - `command`: The command to execute.
    /// - `args`: A slice of arguments for the command.
    ///
    /// # Returns
    /// - `Ok(String)`: The standard output of the command if successful.
    /// - `Err(String)`: An error message if execution fails.
    pub fn execute(method: &str, command: &str, args: &[&str]) -> Result<String, String> {
        debug!("Dispatching execution method: `{}`", method);

        match method {
            "debug" => Self::execute_direct_binary(command, args),
            "execv" => Self::execute_with_execv(command, args),
            "libc" => Self::execute_with_libc(command, args),
            "shell" => Self::execute_with_process(command, args, true),
            "direct" => Self::execute_with_process(command, args, false),
            "check" => match Self::check_command_exists(command) {
                Ok(exists) => Ok(format!("Command `{}` exists: {}", command, exists)),
                Err(e) => Err(e),
            },
            _ => Err(format!("Invalid execution method: {}", method)),
        }
    }

    /// Executes a command using `libc` system calls.
    fn execute_with_libc(command: &str, args: &[&str]) -> Result<String, String> {
        let full_command = Self::build_command_string(command, args)?;
        let c_command = CString::new(full_command)
            .map_err(|e| format!("Failed to construct CString for command: {}", e))?;

        unsafe {
            let status = libc::system(c_command.as_ptr());
            if status == -1 {
                return Err("libc::system call failed.".to_string());
            }

            if libc::WIFEXITED(status) && libc::WEXITSTATUS(status) == 0 {
                Ok("Command executed successfully.".to_string())
            } else {
                Err(format!(
                    "Command failed with exit code: {}",
                    libc::WEXITSTATUS(status)
                ))
            }
        }
    }
    /// Executes a command using `nix::unistd::fork` and `nix::unistd::execv`.
    ///
    /// The command is executed in a child process, allowing the parent process to continue running.
    ///
    /// # Arguments
    /// - `command`: The command to execute (e.g., "/bin/ls").
    /// - `args`: A slice of arguments for the command (e.g., `["-l", "/"]`).
    ///
    /// # Returns
    /// - `Ok(String)`: The output of the command if successful.
    /// - `Err(String)`: An error message if execution fails.
    fn execute_with_execv(command: &str, args: &[&str]) -> Result<String, String> {
        let (c_command, c_args) = Self::convert_to_cstrings(command, args)?;

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child }) => {
                // Parent process: Wait for the child process to finish
                match waitpid(child, None) {
                    Ok(WaitStatus::Exited(_, exit_code)) => {
                        if exit_code == 0 {
                            Ok("Child process executed successfully.".to_string())
                        } else {
                            Err(format!("Child process exited with code: {}", exit_code))
                        }
                    }
                    Ok(WaitStatus::Signaled(_, signal, _)) => {
                        // Convert the signal to a human-readable format using Debug
                        match Signal::try_from(signal) {
                            Ok(signal_enum) => Err(format!(
                                "Child process terminated by signal: {:?}",
                                signal_enum
                            )),
                            Err(_) => Err(format!(
                                "Child process terminated by unknown signal: {:?}",
                                signal
                            )),
                        }
                    }
                    Err(e) => Err(format!("Failed to wait for child process: {}", e)),
                    _ => Err("Unexpected waitpid result.".to_string()),
                }
            }
            Ok(ForkResult::Child) => {
                // Child process: Replace the process image with the new command
                execv(&c_command, &c_args).unwrap_or_else(|e| {
                    error!("Failed to execute command in child process: {}", e);
                    std::process::exit(1); // Exit with an error if execv fails
                });
                unreachable!("execv should not return on success");
            }
            Err(e) => Err(format!("Fork failed: {}", e)),
        }
    }

    /// Executes a command using `std::process::Command`.
    ///
    /// This function supports both shell-based and direct execution modes.
    ///
    /// # Arguments
    /// - `command`: The command to execute.
    /// - `args`: A slice of arguments for the command.
    /// - `use_shell`: Whether to use a shell (`sh -c`) for execution.
    ///
    /// # Returns
    /// - `Ok(String)`: The standard output of the command if successful.
    /// - `Err(String)`: An error message if execution fails.
    fn execute_with_process(
        command: &str,
        args: &[&str],
        use_shell: bool,
    ) -> Result<String, String> {
        let mut cmd = if use_shell {
            // For shell-based execution, construct the command string and use "sh -c"
            let full_command = Self::build_command_string(command, args)?;
            debug!("Executing with shell: `{}`", full_command);
            let mut c = Command::new("sh");
            c.arg("-c").arg(full_command);
            c
        } else {
            // For direct execution, construct the command without using a shell
            debug!(
                "Executing binary directly: `{}` with args: {:?}",
                command, args
            );
            let mut c = Command::new(command);
            for arg in args {
                c.arg(arg);
            }
            c
        };

        let output = cmd
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to execute command: {}", e))?;

        if output.status.success() {
            Ok(Self::convert_to_string(output.stdout))
        } else {
            Err(Self::convert_to_string(output.stderr))
        }
    }

    fn execute_direct_binary(command: &str, args: &[&str]) -> Result<String, String> {
        let mut cmd = Command::new(command);
        for arg in args {
            cmd.arg(arg);
        }

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to execute binary: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    /// Checks if a command exists in the filesystem.
    fn check_command_exists(command: &str) -> Result<bool, String> {
        let path = format!("/bin/{}", command);
        Ok(std::fs::metadata(&path).is_ok())
    }

    // --- Helper Functions ---

    /// Builds a command string from the base command and arguments.
    fn build_command_string(command: &str, args: &[&str]) -> Result<String, String> {
        let escaped_args: Vec<String> = args.iter().map(|arg| Self::shell_escape(arg)).collect();
        Ok(format!("{} {}", command, escaped_args.join(" ")))
    }

    /// Converts a command and arguments into C-compatible strings.
    fn convert_to_cstrings(
        command: &str,
        args: &[&str],
    ) -> Result<(CString, Vec<CString>), String> {
        let c_command = CString::new(command)
            .map_err(|e| format!("Failed to convert command to CString: {}", e))?;
        let c_args = args
            .iter()
            .map(|&arg| {
                CString::new(arg).map_err(|e| format!("Failed to convert arg to CString: {}", e))
            })
            .collect::<Result<Vec<CString>, String>>()?;
        Ok((c_command, c_args))
    }

    /// Escapes shell arguments to handle special characters.
    fn shell_escape(arg: &str) -> String {
        if arg
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            arg.to_string() // No escaping needed
        } else {
            format!("'{}'", arg.replace('\'', r"'\''")) // Escape single quotes
        }
    }

    /// Converts a vector of bytes into a UTF-8 string, handling invalid data.
    fn convert_to_string(output: Vec<u8>) -> String {
        String::from_utf8(output).unwrap_or_else(|_| "<Invalid UTF-8 Output>".to_string())
    }
}
