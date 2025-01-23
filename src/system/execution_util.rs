use std::ffi::CString;
use std::fs::File;
use std::io::{self, Read};
use std::os::fd::FromRawFd;
use std::process::{Command, Stdio};
use std::string::FromUtf8Error;
use nix::unistd::{close, pipe};
use nix::sys::wait::waitpid;
use nix::unistd::{fork, ForkResult};
use nix::sys::wait::WaitStatus;
use log::{debug, error};

/// A static utility class for executing commands in various ways.
pub struct ExecutionUtil;

#[allow(dead_code)]
impl ExecutionUtil {
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
                error!("Failed to execute command: `{}` due to error: {}", command, e);
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
        debug!("Executing command with libc: `{}` with args: {:?}", command, args);

        // Create pipes for stdout and stderr
        let (stdout_read, stdout_write) = pipe().map_err(|e| e.to_string())?;
        let (stderr_read, stderr_write) = pipe().map_err(|e| e.to_string())?;

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child }) => {
                // Parent process: close the write ends of the pipes
                close(stdout_write).map_err(|e| e.to_string())?;
                close(stderr_write).map_err(|e| e.to_string())?;

                // Read stdout and stderr from the pipes
                let mut stdout = String::new();
                let mut stderr = String::new();
                unsafe {
                    let _ = io::BufReader::new(File::from_raw_fd(stdout_read))
                        .read_to_string(&mut stdout);
                    let _ = io::BufReader::new(File::from_raw_fd(stderr_read))
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
                    libc::dup2(stdout_write, libc::STDOUT_FILENO);
                    libc::dup2(stderr_write, libc::STDERR_FILENO);
                }

                // Close the read ends of the pipes
                let _ = close(stdout_read);
                let _ = close(stderr_read);

                // Prepare the command and arguments
                let c_command = CString::new(command).map_err(|e| e.to_string())?;
                let c_args: Vec<CString> = args
                    .iter()
                    .map(|&arg| CString::new(arg).map_err(|e| e.to_string()))
                    .collect::<Result<_, _>>()?;
                let c_args_ptrs: Vec<*const i8> =
                    c_args.iter().map(|s| s.as_ptr()).chain(Some(std::ptr::null())).collect();

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

        Err(format!("Command `{}` does not exist at path: {}", command, path))
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
