use crate::data::models::{EsxiCoreDetail, EsxiCpuDetail, EsxiSystemDto};
#[cfg(unix)]
use std::os::unix::io::AsRawFd;
use std::process::{Command, Output, Stdio};
use std::str;
use std::sync::OnceLock;

/// Static utility for ESXi-specific operations.
pub struct EsxiUtil;

static CACHED_CPU_INFO: OnceLock<String> = OnceLock::new();
static CACHED_CPU_LIST: OnceLock<Vec<String>> = OnceLock::new();
static CACHED_TJMAX: OnceLock<i32> = OnceLock::new();

impl EsxiUtil {
    #[cfg(unix)]
    pub fn is_tty() -> bool {
        unsafe { libc::isatty(std::io::stdout().as_raw_fd()) != 0 }
    }

    #[cfg(not(unix))]
    pub fn is_tty() -> bool {
        // Default behavior for non-Unix systems
        false
    }

    /// Executes a command and handles output, errors, and logging in one place.
    fn execute_command(command: &str, args: &[&str]) -> Result<Output, String> {
        match Command::new(command)
            .args(args)
            .stdin(Stdio::null()) // Prevent TTY expectations for input
            .stdout(Stdio::piped()) // Capture stdout
            .stderr(Stdio::piped()) // Capture stderr
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    Ok(output)
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    log::error!(
                        "Command `{}` with args {:?} failed: {}",
                        command,
                        args,
                        stderr
                    );
                    Err(stderr.to_string())
                }
            }
            Err(e) => {
                log::error!(
                    "Failed to execute command `{}` with args {:?}: {}",
                    command,
                    args,
                    e
                );
                Err(e.to_string())
            }
        }
    }

    /// Checks if the system is running on ESXi by verifying the presence of the `vsish` command.
    pub fn is_running_on_esxi() -> bool {
        Self::execute_command("which", &["vsish"]).is_ok()
    }

    /// Retrieves and caches the TjMax value for the system.
    pub fn get_tjmax() -> i32 {
        *CACHED_TJMAX.get_or_init(|| {
            match Self::execute_command("vsish", &["-e", "cat", "/hardware/msr/pcpu/0/addr/0x1A2"])
            {
                Ok(output) => {
                    let raw_tjmax = str::from_utf8(&output.stdout).unwrap_or("").trim();
                    if Self::validate_hex(raw_tjmax) {
                        if let Ok(raw_value) = i32::from_str_radix(&raw_tjmax[2..], 16) {
                            return (raw_value >> 16) & 0xFF;
                        }
                    }
                    log::warn!("Invalid TjMax value received; using default.");
                    100
                }
                Err(_) => {
                    log::warn!("Failed to retrieve TjMax value; using default.");
                    100
                }
            }
        })
    }

    /// Retrieves and caches CPU topology information: number of sockets, cores, and threads.
    pub fn get_cpu_topology() -> (i32, i32, i32) {
        let cpu_info = Self::get_cached_cpu_info();
        let sockets = Self::parse_topology_value(&cpu_info, "Number of packages");
        let cores = Self::parse_topology_value(&cpu_info, "Number of cores");
        let threads = Self::parse_topology_value(&cpu_info, "Number of CPUs (threads)");
        (sockets, cores, threads)
    }

    /// Retrieves and caches CPU information.
    fn get_cached_cpu_info() -> &'static str {
        CACHED_CPU_INFO.get_or_init(|| {
            match Self::execute_command("vsish", &["-e", "cat", "/hardware/cpu/cpuInfo"]) {
                Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
                Err(_) => String::new(),
            }
        })
    }

    /// Retrieves and caches the CPU list.
    fn get_cached_cpu_list() -> &'static Vec<String> {
        CACHED_CPU_LIST.get_or_init(|| {
            match Self::execute_command("vsish", &["-e", "ls", "/hardware/msr/pcpu/"]) {
                Ok(output) => String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .map(|line| line.trim_end_matches('/').to_string())
                    .collect(),
                Err(_) => Vec::new(),
            }
        })
    }

    /// Helper to parse values from the CPU topology.
    fn parse_topology_value(cpu_info: &str, key: &str) -> i32 {
        cpu_info
            .lines()
            .find(|line| line.contains(key))
            .and_then(|line| line.split(':').nth(1))
            .and_then(|value| value.trim().parse::<i32>().ok())
            .unwrap_or(0)
    }

    /// Retrieves CPU temperature for a specific core.
    pub fn get_cpu_temperature(cpu: &str, tjmax: i32) -> (String, String) {
        let path = format!("/hardware/msr/pcpu/{}/addr/0x19C", cpu);
        match Self::execute_command("vsish", &["-e", "cat", &path]) {
            Ok(output) => {
                let raw_value = str::from_utf8(&output.stdout).unwrap_or("").trim();
                if Self::validate_hex(raw_value) {
                    if let Ok(raw_value_dec) = i32::from_str_radix(&raw_value[2..], 16) {
                        let digital_readout = (raw_value_dec >> 16) & 0x7F;
                        let temperature = tjmax - digital_readout;
                        return (digital_readout.to_string(), temperature.to_string());
                    }
                }
                ("N/A".to_string(), "Invalid temperature value".to_string())
            }
            Err(_) => ("N/A".to_string(), "Error reading MSR".to_string()),
        }
    }

    /// Builds the complete `EsxiSystemDto` for the system using cached data.
    pub fn build_esxi_system_dto() -> EsxiSystemDto {
        let tjmax = Self::get_tjmax();
        let (sockets, cores, threads) = Self::get_cpu_topology();
        let cores_per_socket = cores / sockets.max(1); // Avoid division by zero
        let threads_per_core = threads / cores.max(1);

        let mut cpus = vec![];
        for cpu in Self::get_cached_cpu_list() {
            let core_details = Self::get_core_details(cpu, tjmax);
            let (_, socket) = Self::get_core_socket_info(cpu);

            cpus.push(EsxiCpuDetail {
                cpu_id: cpu.to_string(),
                socket_id: socket,
                cores: core_details,
            });
        }

        EsxiSystemDto {
            tjmax,
            sockets,
            cores_per_socket,
            threads_per_core,
            logical_processors: threads,
            cpus,
        }
    }

    /// Retrieves core details for a specific CPU.
    pub fn get_core_details(cpu: &str, tjmax: i32) -> Vec<EsxiCoreDetail> {
        let mut core_details = vec![];
        let (core, _) = Self::get_core_socket_info(cpu);

        let (digital_readout, temperature) = Self::get_cpu_temperature(cpu, tjmax);
        let core_type = if core == "N/A" {
            "Unknown"
        } else {
            "Real Core"
        };

        core_details.push(EsxiCoreDetail {
            core_id: core,
            temperature,
            digital_readout,
            core_type: core_type.to_string(),
        });

        core_details
    }

    /// Retrieves core and socket information for a specific CPU.
    pub fn get_core_socket_info(cpu: &str) -> (String, String) {
        let path = format!("/hardware/cpu/cpuList/{}", cpu);
        match Self::execute_command("vsish", &["-e", "cat", &path]) {
            Ok(output) => {
                let core_info = str::from_utf8(&output.stdout).unwrap_or("");
                let core = Self::parse_core_socket_info(core_info, "core:");
                let socket = Self::parse_core_socket_info(core_info, "package:");
                (core, socket)
            }
            Err(_) => ("N/A".to_string(), "N/A".to_string()),
        }
    }

    /// Helper to parse core or socket information.
    fn parse_core_socket_info(info: &str, key: &str) -> String {
        info.lines()
            .find(|line| line.to_lowercase().contains(key))
            .and_then(|line| line.split(':').nth(1))
            .unwrap_or("N/A")
            .trim()
            .to_string()
    }

    /// Validates a hexadecimal input string.
    fn validate_hex(input: &str) -> bool {
        input.starts_with("0x") && input[2..].chars().all(|c| c.is_digit(16))
    }
}