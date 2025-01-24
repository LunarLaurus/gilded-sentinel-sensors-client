//! Main Loop Logic
//!
//! This module handles the main application loop, detecting the runtime environment (ESXi or Linux)
//! and delegating to the appropriate environment-specific loop.
#![cfg(unix)]

use crate::config::config_instance::Config;
use crate::config::AppConfig;
use crate::hardware::system_information_monitor::SysInfoMonitor;
use crate::sensor::sensor_util::SensorUtils;
use crate::system::installer::InstallerUtil;
use log::{error, info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Detects the environment and delegates execution to the appropriate loop.
pub fn run_main_loop(running: &Arc<AtomicBool>) {
    info!("System detected as running on Linux.");
    run_linux_main_loop(running, Config::get());
}

/// Main loop for Linux/Dev systems.
fn run_linux_main_loop(running: &Arc<AtomicBool>, config: &AppConfig) {
    if !InstallerUtil::ensure_sensors_installed() {
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
