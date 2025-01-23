use log::{debug, error};
use nix::sys::wait::waitpid;
use nix::sys::wait::WaitStatus;
use nix::unistd::{close, pipe};
use nix::unistd::{fork, ForkResult};
use std::ffi::CString;
use std::fs::File;
use std::io::{self, Read};
use std::os::fd::AsRawFd;
use std::os::fd::FromRawFd;
use std::process::{Command, Stdio};
use std::string::FromUtf8Error;

/// A static utility class for executing commands in various ways.
pub struct ExecutionUtil;

#[allow(dead_code)]
impl ExecutionUtil {
    /// Executes a command without relying on `fork()`, using `libc::system`.
    ///
    /// This is specifically designed for environments like ESXi where `fork()` is unavailable.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to execute (e.g., "ls").
    /// * `args` - A slice of arguments for the command (e.g., `["-l", "/"]`).
    ///
    /// # Returns
    ///
    /// * `Ok(String)` containing the command's standard output if it succeeds.
    /// * `Err(String)` containing the command's standard error or an execution error message if it fails.
    pub fn execute_no_fork(command: &str, args: &[&str]) -> Result<String, String> {
        debug!(
            "Executing command (no fork): `{}` with args: {:?}",
            command, args
        );

        // Manually escape each argument
        let escaped_args: Vec<String> = args.iter().map(|arg| Self::shell_escape(arg)).collect();
        let full_command = format!("{} {}", command, escaped_args.join(" "));

        // Convert the full command into a C-compatible string
        let c_command = match CString::new(full_command) {
            Ok(cstr) => cstr,
            Err(e) => {
                error!("Failed to construct CString for command: {}", e);
                return Err(format!("Invalid command string: {}", e));
            }
        };

        unsafe {
            // Use libc::system to execute the command
            let status = libc::system(c_command.as_ptr());

            if status == -1 {
                let err = "libc::system call failed.";
                error!("{}", err);
                Err(err.to_string())
            } else if libc::WIFEXITED(status) && libc::WEXITSTATUS(status) == 0 {
                debug!("Command executed successfully.");
                Ok("Command executed successfully.".to_string())
            } else {
                let exit_code = libc::WEXITSTATUS(status);
                let err = format!("Command failed with exit code: {}", exit_code);
                error!("{}", err);
                Err(err)
            }
        }
    }

    /// Manual shell argument escaping to handle special characters.
    fn shell_escape(arg: &str) -> String {
        if arg
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            arg.to_string() // No escaping needed
        } else {
            format!("'{}'", arg.replace('\'', r"'\''")) // Wrap in single quotes and escape inner quotes
        }
    }

    /// Executes a command using `std::process::Command`, capturing output and avoiding TTY assumptions.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to execute.
    /// * `args` - A slice of arguments for the command.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` containing the command's standard output if it succeeds.
    /// * `Err(String)` containing the command's standard error or an execution error message if it fails.
    pub fn execute_with_command(command: &str, args: &[&str]) -> Result<String, String> {
        debug!(
            "Executing command with std::process::Command: `{}` with args: {:?}",
            command, args
        );

        let command_str = format!("{} {}", command, args.join(" "));
        let output_result = Command::new("sh")
            .arg("-c")
            .arg(&command_str)
            .stdin(Stdio::null()) // Prevent TTY input
            .stdout(Stdio::piped()) // Capture stdout
            .stderr(Stdio::piped()) // Capture stderr
            .output();

        match output_result {
            Ok(output) => {
                if output.status.success() {
                    let stdout = Self::convert_to_string(output.stdout);
                    debug!("Command succeeded: `{}`", stdout.trim());
                    Ok(stdout)
                } else {
                    let stderr = Self::convert_to_string(output.stderr);
                    error!("Command failed: `{}`", stderr.trim());
                    Err(stderr)
                }
            }
            Err(e) => {
                error!(
                    "Failed to execute command: `{}` due to error: {}",
                    command, e
                );
                Err(e.to_string())
            }
        }
    }

    /// Executes a command using `libc` system calls, avoiding TTY assumptions and providing lower-level control.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to execute.
    /// * `args` - A slice of arguments for the command.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` containing the command's standard output if it succeeds.
    /// * `Err(String)` containing the command's standard error or an execution error message if it fails.
    pub fn execute_with_libc(command: &str, args: &[&str]) -> Result<String, String> {
        debug!(
            "Executing command with libc: `{}` with args: {:?}",
            command, args
        );

        // Create pipes for stdout and stderr
        let (stdout_read, stdout_write) = pipe().map_err(|e| e.to_string())?;
        let (stderr_read, stderr_write) = pipe().map_err(|e| e.to_string())?;

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child }) => {
                // Parent process: close the write ends of the pipes
                close(stdout_write.as_raw_fd()).map_err(|e| e.to_string())?;
                close(stderr_write.as_raw_fd()).map_err(|e| e.to_string())?;

                // Read stdout and stderr from the pipes
                let mut stdout = String::new();
                let mut stderr = String::new();
                unsafe {
                    let _ = io::BufReader::new(File::from_raw_fd(stdout_read.as_raw_fd()))
                        .read_to_string(&mut stdout);
                    let _ = io::BufReader::new(File::from_raw_fd(stderr_read.as_raw_fd()))
                        .read_to_string(&mut stderr);
                }

                // Wait for the child process to complete
                match waitpid(child, None) {
                    Ok(WaitStatus::Exited(_, status)) if status == 0 => Ok(stdout),
                    Ok(WaitStatus::Exited(_, _)) => Err(stderr),
                    Ok(_) => Err("Unexpected child process state.".to_string()),
                    Err(e) => Err(format!("Failed to wait for child process: {}", e)),
                }
            }
            Ok(ForkResult::Child) => {
                // Child process: redirect stdout and stderr to the pipes
                unsafe {
                    libc::dup2(stdout_write.as_raw_fd(), libc::STDOUT_FILENO);
                    libc::dup2(stderr_write.as_raw_fd(), libc::STDERR_FILENO);
                }

                // Close the read ends of the pipes
                let _ = close(stdout_read.as_raw_fd());
                let _ = close(stderr_read.as_raw_fd());

                // Prepare the command and arguments
                let c_command = CString::new(command).map_err(|e| e.to_string())?;
                let c_args: Vec<CString> = args
                    .iter()
                    .map(|&arg| CString::new(arg).map_err(|e| e.to_string()))
                    .collect::<Result<_, _>>()?;
                let c_args_ptrs: Vec<*const i8> = c_args
                    .iter()
                    .map(|s| s.as_ptr())
                    .chain(Some(std::ptr::null()))
                    .collect();

                // Execute the command
                unsafe { libc::execvp(c_command.as_ptr(), c_args_ptrs.as_ptr()) };

                // If execvp fails, exit the child process
                unsafe { libc::_exit(127) };
            }
            Err(e) => Err(format!("Fork failed: {}", e)),
        }
    }

    /// Executes a command by directly checking the file system or invoking shell commands, avoiding process creation.
    ///
    /// # Arguments
    ///
    /// * `command` - The name of the command to check for.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` if the command exists.
    /// * `Err(String)` if the command does not exist or an error occurs.
    pub fn execute_direct_check(command: &str) -> Result<bool, String> {
        debug!("Checking existence of command: `{}`", command);

        let path = format!("/bin/{}", command);
        if std::fs::metadata(&path).is_ok() {
            debug!("Command `{}` exists at path: {}", command, path);
            return Ok(true);
        }

        Err(format!(
            "Command `{}` does not exist at path: {}",
            command, path
        ))
    }

    /// Helper method to safely convert `Vec<u8>` to `String` while handling potential errors.
    ///
    /// # Arguments
    ///
    /// * `output` - A vector of bytes (`Vec<u8>`) representing the output to convert.
    ///
    /// # Returns
    ///
    /// * A `String` if the conversion is successful.
    /// * A fallback `"<Invalid UTF-8 Output>"` if the conversion fails.
    fn convert_to_string(output: Vec<u8>) -> String {
        match String::from_utf8(output) {
            Ok(s) => s,
            Err(FromUtf8Error { .. }) => "<Invalid UTF-8 Output>".to_string(),
        }
    }
}
