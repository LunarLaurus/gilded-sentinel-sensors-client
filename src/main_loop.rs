use crate::config::AppConfig;
use crate::data::models::EsxiSystemDto;
use crate::hardware::esxi::EsxiUtil;
use crate::hardware::system_information_monitor::SysInfoMonitor;
use crate::network::network_util::NetworkUtil;
use log::{error, info};
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub fn run_main_loop(running: &Arc<AtomicBool>, config: &AppConfig) {
    // Detect environment and delegate to the appropriate loop
    if EsxiUtil::is_running_on_esxi() {
        info!("System detected as running on ESXi.");
        run_esxi_main_loop(&running, &config);
    } else {
        info!("System detected as running on Debian.");
        run_debian_main_loop(&running, &config);
    }
}

fn run_esxi_main_loop(running: &Arc<AtomicBool>, config: &AppConfig) {
    let tjmax: i32 = EsxiUtil::get_tjmax();
    let (sockets, cores, threads) = EsxiUtil::get_cpu_topology();

    info!(
        "ESXi Host Info: TjMax = {}°C, Sockets = {}, Cores = {}, Threads = {}",
        tjmax, sockets, cores, threads
    );

    while running.load(Ordering::Relaxed) {
        // Collect CPU data for all threads
        let esxi_data: EsxiSystemDto = EsxiUtil::build_esxi_system_dto();

        // Send the CPU data DTO to the server
        match NetworkUtil::send_with_retries(&esxi_data, &config.server, 3) {
            Ok(_) => info!("ESXi CPU data sent successfully."),
            Err(e) => error!("Failed to send ESXi CPU data: {}", e),
        }

        // Sleep for the configured interval
        thread::sleep(Duration::from_secs(config.interval_secs));
    }
}

fn run_debian_main_loop(running: &Arc<AtomicBool>, config: &AppConfig) {
    let sensors_installed: Result<(), Box<dyn Error>> = ensure_lm_sensors_installed();

    if sensors_installed.is_ok() {
        let mut monitor: SysInfoMonitor = SysInfoMonitor::new();
        monitor.setup_monitoring();
        while running.load(Ordering::Relaxed) {
            // Process and send sensor data
            NetworkUtil::process_sensor_data(&config.server, &mut monitor);

            // Sleep for the configured interval
            thread::sleep(Duration::from_secs(config.interval_secs));
        }
    }
}

fn ensure_lm_sensors_installed() -> Result<(), Box<dyn Error>> {
    if let Err(e) = crate::system::installer::ensure_sensors_installed() {
        error!("Error ensuring lm-sensors package is installed: {}", e);
        return Err(Box::new(e));
    }
    Ok(())
}
