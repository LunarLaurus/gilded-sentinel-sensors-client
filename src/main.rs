// Modules
mod config;
mod data;
mod hardware;
mod network;
mod sensor;
mod system;

// Configuration
use crate::config::config_loader::ConfigLoader;
// Hardware Monitoring
use crate::hardware::system_information_monitor::SysInfoMonitor;
// Networking
use crate::network::network_util::NetworkUtil;
// System Utilities
use crate::system::installer;

use data::models::{CpuInfo, DiskInfo, NetworkInfo};
// Standard Library Imports
use log::{error, info};
use signal_hook_registry::register;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time::Duration};

pub const SIGINT: i32 = 2;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    initialize_logger();
    info!("Starting the Gilded-Sentinel-Debian application...");

    let running = setup_signal_handler()?;
    ensure_lm_sensors_installed()?;

    let config = load_application_config();
    info!(
        "Application running with configuration: server = {}, interval_secs = {}",
        config.server, config.interval_secs
    );

    let mut monitor = SysInfoMonitor::new();
    monitor.setup_monitoring();
    run_main_loop(&running, &config, &mut monitor);

    info!("Shutting down gracefully.");
    Ok(())
}

/// Initializes the logger with default INFO level.
fn initialize_logger() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
}

/// Sets up a signal handler for graceful shutdown.
fn setup_signal_handler() -> Result<Arc<AtomicBool>, Box<dyn std::error::Error>> {
    let running = Arc::new(AtomicBool::new(true));
    let r = Arc::clone(&running);

    unsafe {
        register(SIGINT, move || {
            r.store(false, Ordering::Relaxed);
        })?;
    }

    Ok(running)
}

/// Ensures that the `lm-sensors` package is installed.
fn ensure_lm_sensors_installed() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(e) = installer::ensure_sensors_installed() {
        error!("Error ensuring lm-sensors package is installed: {}", e);
        return Err(Box::new(e));
    }
    Ok(())
}

/// Loads the application configuration.
fn load_application_config() -> config::AppConfig {
    ConfigLoader::new().load_config()
}

/// Runs the main loop for data collection and transmission.
fn run_main_loop(
    running: &Arc<AtomicBool>,
    config: &config::AppConfig,
    monitor: &mut SysInfoMonitor,
) {
    info!("Entering the main loop...");

    while running.load(Ordering::Relaxed) {
        let cpu: CpuInfo = monitor.get_cpu_info();
        let disks: Vec<DiskInfo> = monitor.get_disk_info();
        let networks: Vec<NetworkInfo> = monitor.get_network_info();
        NetworkUtil::process_sensor_data(&config.server, cpu, disks, networks);

        thread::sleep(Duration::from_secs(config.interval_secs));
    }
    info!("Exiting the main loop.");
}

