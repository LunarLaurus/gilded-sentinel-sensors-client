use serde::Serialize;
use std::fmt;
use sysinfo::Component;

// General System DTOs
#[derive(Serialize, Debug)]
pub struct CpuCoreData {
    pub core_name: String,
    pub temperature: f32,
    pub high_threshold: f32,
    pub critical_threshold: f32,
}

#[derive(Serialize, Debug)]
pub struct CpuPackageData {
    pub package_id: String,
    pub adapter_name: String,
    pub package_temperature: f32,
    pub high_threshold: f32,
    pub critical_threshold: f32,
    pub cores: Vec<CpuCoreData>,
}

#[derive(Serialize, Debug)]
pub struct MemoryInfo {
    pub total: u64,
    pub used: u64,
    pub total_swap: u64,
    pub used_swap: u64,
}

#[derive(Serialize, Debug)]
pub struct CpuInfo {
    pub usage_per_core: Vec<f32>,
    pub core_count: usize,
    pub cpu_arch: String,
}

#[derive(Serialize, Debug)]
pub struct DiskInfo {
    pub name: String,
    pub total_space: u64,
    pub available_space: u64,
    pub read_bytes: u64,
    pub written_bytes: u64,
}

#[derive(Serialize, Debug)]
pub struct NetworkInfo {
    pub interface_name: String,
    pub received: u64,
    pub transmitted: u64,
    pub mtu: Option<u64>,
}

#[derive(Serialize, Debug)]
pub struct ProcessInfo {
    pub name: String,
    pub pid: u32,
    pub memory: u64,
}

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
#[derive(Serialize, Debug)]
pub struct SystemInfo {
    pub hostname: String,
    pub uptime: Uptime,
    pub management_ip: String,
}

#[derive(Serialize, Debug)]
pub struct SensorData {
    pub system_info: SystemInfo,
    pub cpu_info: CpuInfo,
    pub cpu_packages: Vec<CpuPackageData>,
    pub memory_info: MemoryInfo,
    pub disks: Vec<DiskInfo>,
    pub network_interfaces: Vec<NetworkInfo>,
    pub components: Vec<ComponentInfo>,
}
