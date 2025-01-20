pub const SERVER: &str = "127.0.0.1:5000"; // Replace with your server's IP and port
const DEFAULT_INTERVAL_SECS: u64 = 5;

pub fn get_interval_from_env() -> u64 {
    std::env::var("SENSOR_INTERVAL")
        .ok()
        .and_then(|val| val.parse::<u64>().ok())
        .unwrap_or(DEFAULT_INTERVAL_SECS)
}
