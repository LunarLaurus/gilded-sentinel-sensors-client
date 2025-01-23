mod config;
mod data;
mod hardware;
mod main_loop;
mod network;
mod sensor;
mod system;

use config::config_loader::{initialize_logger, load_application_config};
use hardware::esxi::EsxiUtil;
use log::{info, warn};
use std::sync::{atomic::AtomicBool, Arc};
use system::signal::setup_signal_handler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    initialize_logger();

    let running: Arc<AtomicBool>;
    let is_tty: bool = EsxiUtil::is_tty();

    if is_tty {
        info!("Running in a Teletype Environment.");
        running = setup_signal_handler()?;
    } else {
        warn!("Not running in a Teletype Environment.");
        running = Arc::new(AtomicBool::new(true));
    }

    info!("Starting the Gilded-Sentinel application.");
    let config: config::AppConfig = load_application_config();

    info!(
        "Application running with configuration: server = {}, interval_secs = {}",
        config.server, config.interval_secs
    );

    info!("Executing Main Loop.");
    main_loop::run_main_loop(&running, &config);

    info!("Shutting down gracefully.");
    Ok(())
}
