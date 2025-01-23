//! Signal Handling
//! 
//! This module handles signal registration for graceful application shutdown.

use signal_hook_registry::register;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub const SIGINT: i32 = 2;

/// Sets up a signal handler for SIGINT (Ctrl+C) to enable graceful shutdown.
pub fn setup_signal_handler() -> Result<Arc<AtomicBool>, Box<dyn std::error::Error>> {
    let running = Arc::new(AtomicBool::new(true));
    let r = Arc::clone(&running);

    unsafe {
        register(SIGINT, move || {
            r.store(false, Ordering::Relaxed);
        })?;
    }

    Ok(running)
}
