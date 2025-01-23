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

use data::models::EsxiCpuDetail;
use hardware::esxi::EsxiUtil;
use log::{error, info};
use signal_hook_registry::register;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time::Duration};

pub const SIGINT: i32 = 2;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    initialize_logger();
    info!("Starting the Gilded-Sentinel application...");

    // Load configuration and set up signal handler
    let config = load_application_config();
    let running = setup_signal_handler()?;

    info!(
        "Application running with configuration: server = {}, interval_secs = {}",
        config.server, config.interval_secs
    );

    // Detect the environment and initialize the appropriate main loop
    if EsxiUtil::is_running_on_esxi() {
        info!("System detected as running on ESXi.");
        run_esxi_main_loop(&running, &config);
    } else {
        info!("System detected as running on Debian.");
        run_debian_main_loop(&running, &config)?;
    }

    info!("Shutting down gracefully.");
    Ok(())
}

/// Runs the main loop for ESXi, sending CPU temperature and info data as DTOs.
fn run_esxi_main_loop(running: &Arc<AtomicBool>, config: &config::AppConfig) {
    let tjmax = EsxiUtil::get_tjmax();
    let (sockets, cores, threads) = EsxiUtil::get_cpu_topology();

    info!(
        "ESXi Host Info: TjMax = {}°C, Sockets = {}, Cores = {}, Threads = {}",
        tjmax, sockets, cores, threads
    );

    while running.load(Ordering::Relaxed) {
        // Collect CPU data for all threads        
        let esxi_data = EsxiUtil::build_esxi_system_dto();

        // Send the CPU data DTO to the server
        match NetworkUtil::send_with_retries(&esxi_data, &config.server, 3) {
            Ok(_) => info!("ESXi CPU data sent successfully."),
            Err(e) => error!("Failed to send ESXi CPU data: {}", e),
        }

        // Sleep for the configured interval
        thread::sleep(Duration::from_secs(config.interval_secs));
    }
}

/// Runs the main loop for Debian, monitoring and sending sensor data as DTOs.
fn run_debian_main_loop(
    running: &Arc<AtomicBool>,
    config: &config::AppConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    ensure_lm_sensors_installed()?;

    let mut monitor = SysInfoMonitor::new();
    monitor.setup_monitoring();

    while running.load(Ordering::Relaxed) {
        // Process and send sensor data
        NetworkUtil::process_sensor_data(&config.server, &mut monitor);

        // Sleep for the configured interval
        thread::sleep(Duration::from_secs(config.interval_secs));
    }

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
    if let Err(e) = crate::system::installer::ensure_sensors_installed() {
        error!("Error ensuring lm-sensors package is installed: {}", e);
        return Err(Box::new(e));
    }
    Ok(())
}

/// Loads the application configuration.
fn load_application_config() -> config::AppConfig {
    ConfigLoader::new().load_config()
}
