//! Main Loop Logic
//!
//! This module handles the main application loop, detecting the runtime environment (ESXi or Linux)
//! and delegating to the appropriate environment-specific loop.

use crate::config::AppConfig;
use crate::hardware::esxi::EsxiUtil;
use crate::hardware::system_information_monitor::SysInfoMonitor;
use crate::network::network_util::NetworkUtil;
use crate::sensor::sensor_util::SensorUtils;
use crate::system::installer::ensure_sensors_installed;
use log::{error, info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Detects the environment and delegates execution to the appropriate loop.
pub fn run_main_loop(running: &Arc<AtomicBool>, config: &AppConfig) {
    info!("Detecting environment and delegating.");
    if EsxiUtil::is_running_on_esxi() {
        info!("System detected as running on ESXi.");
        run_esxi_main_loop(running, config);
    } else {
        info!("System detected as running on Linux.");
        run_linux_main_loop(running, config);
    }
}

/// Main loop for ESXi systems.
fn run_esxi_main_loop(running: &Arc<AtomicBool>, config: &AppConfig) {
    let tjmax = EsxiUtil::get_tjmax();
    let (sockets, cores, threads) = EsxiUtil::get_cpu_topology();

    info!(
        "ESXi Host Info: TjMax = {}°C, Sockets = {}, Cores = {}, Threads = {}",
        tjmax, sockets, cores, threads
    );

    while running.load(Ordering::Relaxed) {
        let esxi_data = EsxiUtil::build_esxi_system_dto();

        match NetworkUtil::send_with_retries(&esxi_data, &config.server, 3) {
            Ok(_) => info!("ESXi CPU data sent successfully."),
            Err(e) => error!("Failed to send ESXi CPU data: {}", e),
        }

        thread::sleep(Duration::from_secs(config.interval_secs));
    }
}

/// Main loop for Linux systems.
fn run_linux_main_loop(running: &Arc<AtomicBool>, config: &AppConfig) {
    if !ensure_sensors_installed() {
        error!("Failed to ensure lm-sensors is installed.");
        return;
    }

    let mut monitor = SysInfoMonitor::new();
    monitor.setup_monitoring();

    while running.load(Ordering::Relaxed) {
        SensorUtils::process_sensor_data(&config.server, &mut monitor);
        thread::sleep(Duration::from_secs(config.interval_secs));
    }
}
