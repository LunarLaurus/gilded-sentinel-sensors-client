//! Gilded-Sentinel-Client Application Entry Point
//!
//! This file serves as the entry point for the Gilded-Sentinel system monitoring tool. It initializes
//! the application, sets up signal handling, and delegates execution to the appropriate main loop
//! based on the environment (e.g., ESXi or Linux).

mod config;
mod data;
mod hardware;
mod main_loop;
mod network;
mod sensor;
mod system;

use config::config_loader::{initialize_logger, load_application_config};
use config::AppConfig;

use log::{info, warn};
use std::sync::{atomic::AtomicBool, Arc};
use system::{signal::setup_signal_handler, system_util::SystemUtil};

/// Main entry point for the Gilded-Sentinel application.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    initialize_logger();

    SystemUtil::redirect_to_null();
    let is_tty: bool = SystemUtil::is_tty();

    let running: Arc<AtomicBool> = if is_tty {
        info!("Running in a Teletype Environment.");
        setup_signal_handler()?
    } else {
        warn!("Not running in a Teletype Environment.");
        Arc::new(AtomicBool::new(true))
    };

    info!("Starting the Gilded-Sentinel-Client application.");
    let config: config::AppConfig = load_application_config();

    info!(
        "Application running with configuration: server = {}, interval_secs = {}",
        config.server, config.interval_secs
    );

    info!("Executing Main Loop.");
    setup(&running, &config);

    info!("Shutting down gracefully.");
    Ok(())
}

#[cfg(unix)]
fn setup(running: &Arc<AtomicBool>, config: &AppConfig) {
    main_loop::run_main_loop(running, config);
}
#[cfg(not(unix))]
fn setup(_running: &Arc<AtomicBool>, _config: &AppConfig) {}
