#[cfg(unix)]
use std::fs::OpenOptions;
#[cfg(unix)]
use std::os::fd::AsRawFd;

#[cfg(not(unix))]
use log::{self, debug};

/// A utility class for interacting with the system.
pub struct SystemUtil;

#[cfg(unix)]
impl SystemUtil {
    /// Checks if the program is running in a TTY environment (Unix-based).
    pub fn is_tty() -> bool {
        unsafe { libc::isatty(std::io::stdout().as_raw_fd()) != 0 }
    }

    /// Redirects input, output, and error streams to `/dev/null` (Unix-based).
    pub fn redirect_to_null() {
        let dev_null = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/null")
            .unwrap();
        unsafe {
            libc::dup2(dev_null.as_raw_fd(), libc::STDIN_FILENO);
            // Uncomment the following if you want to redirect stdout/stderr as well
            // libc::dup2(dev_null.as_raw_fd(), libc::STDOUT_FILENO);
            // libc::dup2(dev_null.as_raw_fd(), libc::STDERR_FILENO);
        }
    }
}

#[cfg(not(unix))]
#[allow(dead_code)]
impl SystemUtil {
    /// Mock for TTY check on non-Unix platforms.
    pub fn is_tty() -> bool {
        debug!("is_tty is a no-op on non-Unix platforms.");
        true
    }

    /// Mock for redirecting streams to `/dev/null` on non-Unix platforms.
    pub fn redirect_to_null() {
        debug!("redirect_to_null is a no-op on non-Unix platforms.");
    }
}
