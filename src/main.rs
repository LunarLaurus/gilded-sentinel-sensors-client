mod config;
mod data;
mod hardware;
mod main_loop;
mod network;
mod sensor;
mod system;

use config::config_loader::{initialize_logger, load_application_config};
use log::info;
use system::signal::setup_signal_handler;

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

    // Detect environment and delegate to the appropriate loop
    if hardware::esxi::EsxiUtil::is_running_on_esxi() {
        info!("System detected as running on ESXi.");
        main_loop::run_esxi_main_loop(&running, &config);
    } else {
        info!("System detected as running on Debian.");
        main_loop::run_debian_main_loop(&running, &config)?;
    }

    info!("Shutting down gracefully.");
    Ok(())
}
