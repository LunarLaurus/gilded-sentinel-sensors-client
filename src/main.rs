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
use hardware::esxi_util::EsxiUtil;
use log::{info, warn};
use std::sync::{atomic::AtomicBool, Arc};
use system::signal::setup_signal_handler;

/// Main entry point for the Gilded-Sentinel application.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    initialize_logger();

    let running: Arc<AtomicBool>;
    EsxiUtil::redirect_to_null();
    let is_tty: bool = EsxiUtil::is_tty();

    if is_tty {
        info!("Running in a Teletype Environment.");
        running = setup_signal_handler()?;
    } else {
        warn!("Not running in a Teletype Environment.");
        running = Arc::new(AtomicBool::new(true));
    }

    info!("Starting the Gilded-Sentinel-Client application.");
    let config = load_application_config();

    info!(
        "Application running with configuration: server = {}, interval_secs = {}",
        config.server, config.interval_secs
    );

    info!("Executing Main Loop.");
    main_loop::run_main_loop(&running, &config);

    info!("Shutting down gracefully.");
    Ok(())
}
