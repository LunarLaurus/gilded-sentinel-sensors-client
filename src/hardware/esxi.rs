#![cfg(unix)]

use crate::data::models::{EsxiCoreDetail, EsxiCpuDetail, EsxiSystemDto};
use std::collections::{HashMap, HashSet};
use std::str;
use std::sync::OnceLock;

use super::esxi_util::EsxiUtil;

/// Static utility for ESXi-specific operations.
pub struct Esxi;

// Caches for performance optimization
static CACHED_CPU_INFO: OnceLock<String> = OnceLock::new();
static CACHED_CPU_LIST: OnceLock<Vec<String>> = OnceLock::new();
static CACHED_CORE_TYPES: OnceLock<HashMap<String, String>> = OnceLock::new();
static CACHED_TJMAX: OnceLock<i32> = OnceLock::new();

impl Esxi {
    // -----------------------------------
    // CPU Information Retrieval
    // -----------------------------------

    /// Retrieves and caches the TjMax value for the system.
    pub fn get_tjmax() -> i32 {
        *CACHED_TJMAX.get_or_init(|| {
            match EsxiUtil::execute_command(
                "vsish",
                &["-e", "cat", "/hardware/msr/pcpu/0/addr/0x1A2"],
            ) {
                Ok(output) => {
                    let raw_tjmax = output.trim();
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
        let sockets = Self::parse_topology_value(cpu_info, "Number of packages");
        let cores = Self::parse_topology_value(cpu_info, "Number of cores");
        let threads = Self::parse_topology_value(cpu_info, "Number of CPUs (threads)");
        (sockets, cores, threads)
    }

    /// Retrieves and caches CPU information.
    fn get_cached_cpu_info() -> &'static str {
        CACHED_CPU_INFO.get_or_init(|| {
            EsxiUtil::execute_command("vsish", &["-e", "cat", "/hardware/cpu/cpuInfo"]).unwrap_or_default()
        })
    }

    /// Retrieves and caches the CPU list.
    fn get_cached_cpu_list() -> &'static Vec<String> {
        CACHED_CPU_LIST.get_or_init(|| {
            match EsxiUtil::execute_command("vsish", &["-e", "ls", "/hardware/msr/pcpu/"]) {
                Ok(output) => output
                    .lines()
                    .map(|line| line.trim_end_matches('/').to_string())
                    .collect(),
                Err(_) => Vec::new(),
            }
        })
    }

    /// Retrieves core and socket information for a specific CPU.
    pub fn get_core_socket_info(cpu: &str) -> (String, String) {
        let path = format!("/hardware/cpu/cpuList/{}", cpu);
        match EsxiUtil::execute_command("vsish", &["-e", "cat", &path]) {
            Ok(output) => {
                let core_info = output;
                let core = Self::parse_core_socket_info(&core_info, "core:");
                let socket = Self::parse_core_socket_info(&core_info, "package:");
                (core, socket)
            }
            Err(_) => ("N/A".to_string(), "N/A".to_string()),
        }
    }

    /// Retrieves CPU temperature for a specific core.
    pub fn get_cpu_temperature(cpu: &str, tjmax: i32) -> (String, String) {
        let path = format!("/hardware/msr/pcpu/{}/addr/0x19C", cpu);
        match EsxiUtil::execute_command("vsish", &["-e", "cat", &path]) {
            Ok(output) => {
                let raw_value = output.trim();
                if Self::validate_hex(raw_value) {
                    if let Ok(raw_value_dec) = i32::from_str_radix(&raw_value[2..], 16) {
                        let digital_readout = (raw_value_dec >> 16) & 0x7F;
                        let temperature = tjmax - digital_readout;
                        return (digital_readout.to_string(), temperature.to_string());
                    }
                }
                log::warn!("Invalid temperature value received for CPU {}.", cpu);
                ("N/A".to_string(), "Invalid temperature".to_string())
            }
            Err(_) => {
                log::error!("Failed to retrieve temperature for CPU {}.", cpu);
                ("N/A".to_string(), "Error reading MSR".to_string())
            }
        }
    }

    // -----------------------------------
    // DTO Construction
    // -----------------------------------

    /// Builds the complete `EsxiSystemDto` for the system using cached data.
    pub fn build_esxi_system_dto() -> EsxiSystemDto {
        let tjmax = Self::get_tjmax();
        let (sockets, cores, threads) = Self::get_cpu_topology();
        let cores_per_socket = cores / sockets.max(1); // Avoid division by zero
        let threads_per_core = threads / cores.max(1);

        let mut physical_cores: HashSet<String> = HashSet::new(); // Track physical cores
        let mut core_type_cache: HashMap<String, String> = HashMap::new(); // Cache core types

        let cpus: Vec<EsxiCpuDetail> = Self::get_cached_cpu_list()
            .iter()
            .map(|cpu| {
                let (core, socket) = Self::get_core_socket_info(cpu);

                // Determine core type
                let core_type: String = if physical_cores.contains(&core) {
                    "Virtual Thread".to_string()
                } else {
                    physical_cores.insert(core.clone());
                    "Real Core".to_string()
                };

                core_type_cache.insert(core.clone(), core_type.clone()); // Cache the core type

                let (digital_readout, temperature) = Self::get_cpu_temperature(cpu, tjmax);

                EsxiCpuDetail {
                    cpu_id: cpu.clone(),
                    socket_id: socket,
                    cores: vec![EsxiCoreDetail {
                        core_id: core,
                        temperature,
                        digital_readout,
                        core_type,
                    }],
                }
            })
            .collect();

        // Cache the core types globally
        CACHED_CORE_TYPES.set(core_type_cache).ok();

        EsxiSystemDto {
            tjmax,
            sockets,
            cores_per_socket,
            threads_per_core,
            logical_processors: threads,
            cpus,
        }
    }

    // -----------------------------------
    // Helper Functions
    // -----------------------------------

    /// Helper to parse topology values from the CPU info.
    fn parse_topology_value(cpu_info: &str, key: &str) -> i32 {
        cpu_info
            .lines()
            .find(|line| line.contains(key))
            .and_then(|line| line.split(':').nth(1))
            .and_then(|value| value.trim().parse::<i32>().ok())
            .unwrap_or(0)
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
        input.starts_with("0x") && input[2..].chars().all(|c| c.is_ascii_hexdigit())
    }
}
