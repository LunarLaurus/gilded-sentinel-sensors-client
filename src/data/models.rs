use serde::Serialize;
use std::fmt;
use sysinfo::Component;

/// Represents individual CPU core data.
#[derive(Serialize, Debug)]
pub struct CpuCoreData {
    pub core_name: String,
    pub temperature: f32,
    pub high_threshold: f32,
    pub critical_threshold: f32,
}

/// Represents CPU package data.
#[derive(Serialize, Debug)]
pub struct CpuPackageData {
    pub package_id: String,
    pub adapter_name: String,
    pub package_temperature: f32,
    pub high_threshold: f32,
    pub critical_threshold: f32,
    pub cores: Vec<CpuCoreData>,
}

/// Represents memory usage information.
#[derive(Serialize, Debug)]
pub struct MemoryInfo {
    pub total: u64,
    pub used: u64,
    pub total_swap: u64,
    pub used_swap: u64,
}

/// Represents CPU information, including usage and architecture.
#[derive(Serialize, Debug)]
pub struct CpuInfo {
    pub usage_per_core: Vec<f32>,
    pub core_count: usize,
    pub cpu_arch: String,
}

/// Represents disk usage information.
#[derive(Serialize, Debug)]
pub struct DiskInfo {
    pub name: String,
    pub total_space: u64,
    pub available_space: u64,
    pub read_bytes: u64,
    pub written_bytes: u64,
}

/// Represents network usage information.
#[derive(Serialize, Debug)]
pub struct NetworkInfo {
    pub interface_name: String,
    pub received: u64,
    pub transmitted: u64,
    pub mtu: Option<u64>,
}

/// Represents information about a single process.
#[derive(Serialize, Debug)]
pub struct ProcessInfo {
    pub name: String,
    pub pid: u32,
    pub memory: u64,
}

/// Represents detailed information about a system component.
#[derive(Serialize, Debug)]
pub struct ComponentInfo {
    pub label: String,
    pub temperature: Option<f32>,
    pub max_temperature: Option<f32>,
    pub critical_temperature: Option<f32>,
}

impl From<&Component> for ComponentInfo {
    fn from(component: &Component) -> Self {
        Self {
            label: component.label().to_string(),
            temperature: component.temperature(),
            max_temperature: component.max(),
            critical_temperature: component.critical(),
        }
    }
}

/// Represents system uptime.
#[derive(Serialize, Debug)]
pub struct Uptime {
    pub days: u64,
    pub hours: u64,
    pub minutes: u64,
    pub seconds: u64,
    pub total_seconds: u64,
}

impl Uptime {
    pub fn from_seconds(total_seconds: u64) -> Self {
        let days = total_seconds / 86400;
        let hours = (total_seconds % 86400) / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        Self {
            days,
            hours,
            minutes,
            seconds,
            total_seconds,
        }
    }

    pub fn to_string(&self) -> String {
        format!(
            "{} days {} hours {} minutes {} seconds [{}]",
            self.days, self.hours, self.minutes, self.seconds, self.total_seconds
        )
    }
}

impl fmt::Display for Uptime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} days {} hours {} minutes {} seconds",
            self.days, self.hours, self.minutes, self.seconds
        )
    }
}

/// Represents the complete sensor data for the system, including all relevant DTOs.
#[derive(Serialize, Debug)]
pub struct SensorData {
    pub uptime: Uptime,
    pub cpu_info: CpuInfo,
    pub cpu_packages: Vec<CpuPackageData>,
    pub memory_info: MemoryInfo,
    pub disks: Vec<DiskInfo>,
    pub network_interfaces: Vec<NetworkInfo>,
    pub components: Vec<ComponentInfo>,
}
