use std::process::Command;
use std::str;

/// Utility for detecting ESXi and retrieving system information.
pub struct EsxiUtil;

impl EsxiUtil {
    /// Checks if the system is running on ESXi by testing for the `vsish` command.
    pub fn is_running_on_esxi() -> bool {
        Command::new("which")
            .arg("vsish")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Reads and calculates the TjMax value for ESXi.
    pub fn get_tjmax() -> i32 {
        if let Ok(output) = Command::new("vsish")
            .args(&["-e", "cat", "/hardware/msr/pcpu/0/addr/0x1A2"])
            .output()
        {
            let raw_tjmax = str::from_utf8(&output.stdout).unwrap_or("").trim();
            if Self::validate_hex(raw_tjmax) {
                let raw_value = i32::from_str_radix(&raw_tjmax[2..], 16).unwrap_or(0);
                return (raw_value >> 16) & 0xFF;
            }
        }
        100 // Default TjMax value
    }

    /// Retrieves CPU topology information from ESXi.
    pub fn get_cpu_topology() -> (i32, i32, i32) {
        if let Ok(output) = Command::new("vsish")
            .args(&["-e", "cat", "/hardware/cpu/cpuInfo"])
            .output()
        {
            let cpu_info = str::from_utf8(&output.stdout).unwrap_or("");
            let sockets = cpu_info
                .lines()
                .find(|line| line.contains("Number of packages"))
                .and_then(|line| line.split(':').nth(1))
                .and_then(|value| value.trim().parse::<i32>().ok())
                .unwrap_or(0);

            let cores = cpu_info
                .lines()
                .find(|line| line.contains("Number of cores"))
                .and_then(|line| line.split(':').nth(1))
                .and_then(|value| value.trim().parse::<i32>().ok())
                .unwrap_or(0);

            let threads = cpu_info
                .lines()
                .find(|line| line.contains("Number of CPUs (threads)"))
                .and_then(|line| line.split(':').nth(1))
                .and_then(|value| value.trim().parse::<i32>().ok())
                .unwrap_or(0);

            return (sockets, cores, threads);
        }
        (0, 0, 0) // Default values
    }

    /// Retrieves core and socket information for a specific CPU.
    pub fn get_core_socket_info(cpu: &str) -> (String, String) {
        let path = format!("/hardware/cpu/cpuList/{}", cpu);
        if let Ok(output) = Command::new("vsish")
            .args(&["-e", "cat", &path])
            .output()
        {
            let core_info = str::from_utf8(&output.stdout).unwrap_or("");
            let core = core_info
                .lines()
                .find(|line| line.to_lowercase().contains("core:"))
                .and_then(|line| line.split(':').nth(1))
                .unwrap_or("N/A")
                .trim()
                .to_string();

            let socket = core_info
                .lines()
                .find(|line| line.to_lowercase().contains("package:"))
                .and_then(|line| line.split(':').nth(1))
                .unwrap_or("N/A")
                .trim()
                .to_string();

            return (core, socket);
        }
        ("N/A".to_string(), "N/A".to_string())
    }

    /// Retrieves CPU temperature for a specific core.
    pub fn get_cpu_temperature(cpu: &str, tjmax: i32) -> (String, String) {
        let path = format!("/hardware/msr/pcpu/{}/addr/0x19C", cpu);
        if let Ok(output) = Command::new("vsish")
            .args(&["-e", "cat", &path])
            .output()
        {
            let raw_value = str::from_utf8(&output.stdout).unwrap_or("").trim();
            if Self::validate_hex(raw_value) {
                let raw_value_dec = i32::from_str_radix(&raw_value[2..], 16).unwrap_or(0);
                let digital_readout = (raw_value_dec >> 16) & 0x7F;
                let temperature = tjmax - digital_readout;
                return (digital_readout.to_string(), temperature.to_string());
            }
        }
        ("N/A".to_string(), "Error reading MSR".to_string())
    }

    /// Validates a hexadecimal input string.
    fn validate_hex(input: &str) -> bool {
        input.starts_with("0x") && input[2..].chars().all(|c| c.is_digit(16))
    }
}
