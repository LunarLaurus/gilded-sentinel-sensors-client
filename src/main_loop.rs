use crate::config::AppConfig;
use crate::hardware::esxi::EsxiUtil;
use crate::hardware::system_information_monitor::SysInfoMonitor;
use crate::network::network_util::NetworkUtil;
use log::{error, info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub fn run_esxi_main_loop(running: &Arc<AtomicBool>, config: &AppConfig) {
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

pub fn run_debian_main_loop(
    running: &Arc<AtomicBool>,
    config: &AppConfig,
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

fn ensure_lm_sensors_installed() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(e) = crate::system::installer::ensure_sensors_installed() {
        error!("Error ensuring lm-sensors package is installed: {}", e);
        return Err(Box::new(e));
    }
    Ok(())
}
