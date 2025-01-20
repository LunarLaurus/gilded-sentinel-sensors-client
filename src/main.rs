use std::{
    io::{self, Write},
    net::TcpStream,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

// Constants for server and default interval
const SERVER: &str = "127.0.0.1:5000"; // Replace with your server's IP and port
const DEFAULT_INTERVAL_SECS: u64 = 5; // Default interval to check sensor data

fn main() {
    // Read the interval for data collection from environment or use the default
    let interval_secs = get_interval_from_env();

    println!("Reading sensor data every {} seconds...", interval_secs);

    // Start the main loop for data collection and transmission
    start_data_collection(interval_secs);
}

/// Reads the interval from the `SENSOR_INTERVAL` environment variable.
/// Falls back to a default value if the variable is not set or invalid.
fn get_interval_from_env() -> u64 {
    std::env::var("SENSOR_INTERVAL")
        .ok()
        .and_then(|val| val.parse::<u64>().ok())
        .unwrap_or(DEFAULT_INTERVAL_SECS)
}

/// Starts the main loop to collect and send sensor data at the given interval.
fn start_data_collection(interval_secs: u64) {
    loop {
        // Retrieve sensor data (mocked on Windows, real on Linux)
        let sensor_data = collect_sensor_data();

        // Send the sensor data to the specified server
        match send_data_to_server(&sensor_data, SERVER) {
            Ok(_) => println!("Sensor data sent successfully."),
            Err(e) => eprintln!("Error sending data over socket: {}", e),
        }

        // Wait for the specified interval before the next iteration
        thread::sleep(Duration::from_secs(interval_secs));
    }
}

/// Collects sensor data based on the operating system.
/// - On Windows: Returns mock data.
/// - On Linux: Executes the `sensors` command and retrieves its output.
fn collect_sensor_data() -> String {
    if cfg!(target_os = "windows") {
        get_mock_sensor_data()
    } else {
        match execute_sensors_command() {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Error retrieving sensor data: {}", e);
                String::new() // Return empty data on error
            }
        }
    }
}

/// Mock sensor data for Windows (sample data provided).
fn get_mock_sensor_data() -> String {
    r#"sensors
    tg3-pci-0300
    Adapter: PCI adapter
    temp1:        +55.0°C  (high = +100.0°C, crit = +110.0°C)
    
    coretemp-isa-0002
    Adapter: ISA adapter
    Package id 2:  +29.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 0:        +19.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 1:        +23.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 2:        +18.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 3:        +26.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 4:        +25.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 8:        +18.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 9:        +22.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 10:       +23.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 11:       +30.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 12:       +23.0°C  (high = +76.0°C, crit = +86.0°C)
    
    coretemp-isa-0000
    Adapter: ISA adapter
    Package id 0:  +29.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 0:        +22.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 1:        +25.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 2:        +25.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 3:        +24.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 4:        +24.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 8:        +24.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 9:        +28.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 10:       +25.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 11:       +25.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 12:       +27.0°C  (high = +76.0°C, crit = +86.0°C)
    
    acpitz-acpi-0
    Adapter: ACPI interface
    temp1:         +8.3°C
    
    coretemp-isa-0003
    Adapter: ISA adapter
    Package id 3:  +33.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 0:        +26.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 1:        +29.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 2:        +25.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 3:        +30.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 4:        +32.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 8:        +29.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 9:        +29.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 10:       +28.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 11:       +31.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 12:       +27.0°C  (high = +76.0°C, crit = +86.0°C)
    
    coretemp-isa-0001
    Adapter: ISA adapter
    Package id 1:  +31.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 0:        +25.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 1:        +19.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 2:        +28.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 3:        +27.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 4:        +25.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 8:        +26.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 9:        +28.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 10:       +29.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 11:       +32.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 12:       +23.0°C  (high = +76.0°C, crit = +86.0°C)
    
    power_meter-acpi-0
    Adapter: ACPI interface
    power1:      130.00 W  (interval = 300.00 s)
    "#
    .to_string()
}

/// Executes the `sensors` command on Linux and captures its output.
fn execute_sensors_command() -> io::Result<String> {
    let output = Command::new("sensors")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("sensors command failed: {}", err_msg),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Sends the given data to the specified server over a TCP socket.
fn send_data_to_server(data: &str, server: &str) -> io::Result<()> {
    let mut stream = TcpStream::connect(server)?;
    stream.write_all(data.as_bytes())?;
    Ok(())
}
