use std::fmt;

use sysinfo::{Component, Components, Disks, Networks, Pid, Signal, System, Users};

#[derive(serde::Serialize)]
pub struct MemoryInfo {
    pub total: u64,
    pub used: u64,
    pub total_swap: u64,
    pub used_swap: u64,
}

#[derive(serde::Serialize)]
pub struct CpuInfo {
    pub usage_per_core: Vec<f32>,
    pub core_count: usize,
    pub cpu_arch: String,
}

#[derive(serde::Serialize)]
pub struct DiskInfo {
    pub name: String,
    pub total_space: u64,
    pub available_space: u64,
    pub read_bytes: u64,
    pub written_bytes: u64,
}

#[derive(serde::Serialize)]
pub struct NetworkInfo {
    pub interface_name: String,
    pub received: u64,
    pub transmitted: u64,
    pub mtu: Option<u64>,
}

#[derive(serde::Serialize)]
pub struct ProcessInfo {
    pub name: String,
    pub pid: u32,
    pub memory: u64,
}

pub struct SystemInfo {
    system: System,
    networks: Networks,
    disks: Disks,
    components: Components,
    users: Users,
}

#[derive(serde::Serialize)]
pub struct Uptime {
    pub days: u64,
    pub hours: u64,
    pub minutes: u64,
    pub seconds: u64,
    pub total_seconds: u64,
}
#[derive(serde::Serialize, Debug)]
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

impl SystemInfo {
    /// Initializes a new `SystemInfo` instance and refreshes all data.
    pub fn new() -> Self {
        let mut system = System::new_all();
        let networks = Networks::new_with_refreshed_list();
        let disks = Disks::new_with_refreshed_list();
        let components = Components::new_with_refreshed_list();
        let users = Users::new_with_refreshed_list();

        system.refresh_all();

        Self {
            system,
            networks,
            disks,
            components,
            users,
        }
    }

    /// Refreshes all system data.
    pub fn refresh(&mut self) {
        self.system.refresh_all();
        self.networks.refresh(false);
        self.disks.refresh(false);
        self.components.refresh(false);
        self.users.refresh();
    }

    /// Retrieves the user information as a read-only reference.
    pub fn get_users(&self) -> &Users {
        &self.users
    }

    /// Retrieves the system components as a read-only reference.
    pub fn get_components(&self) -> &Components {
        &self.components
    }

    /// Retrieves memory information.
    pub fn memory_info(&self) -> MemoryInfo {
        MemoryInfo {
            total: self.system.total_memory(),
            used: self.system.used_memory(),
            total_swap: self.system.total_swap(),
            used_swap: self.system.used_swap(),
        }
    }

    /// Retrieves CPU information.
    pub fn cpu_info(&self) -> CpuInfo {
        CpuInfo {
            usage_per_core: self
                .system
                .cpus()
                .iter()
                .map(|cpu| cpu.cpu_usage())
                .collect(),
            core_count: self.system.cpus().len(),
            cpu_arch: sysinfo::System::cpu_arch(),
        }
    }

    /// Retrieves disk information as a vector of `DiskInfo`.
    pub fn disk_info(&self) -> Vec<DiskInfo> {
        self.disks
            .iter()
            .map(|disk| {
                let usage = disk.usage();
                DiskInfo {
                    name: disk.name().to_string_lossy().to_string(),
                    total_space: disk.total_space(),
                    available_space: disk.available_space(),
                    read_bytes: usage.read_bytes,
                    written_bytes: usage.written_bytes,
                }
            })
            .collect()
    }

    /// Retrieves network information as a vector of `NetworkInfo`.
    pub fn network_info(&self) -> Vec<NetworkInfo> {
        self.networks
            .iter()
            .map(|(name, data)| NetworkInfo {
                interface_name: name.clone(),
                received: data.received(),
                transmitted: data.transmitted(),
                mtu: Some(data.mtu()),
            })
            .collect()
    }

    /// Retrieves process information as a vector of `ProcessInfo`.
    pub fn process_info(&self) -> Vec<ProcessInfo> {
        self.system
            .processes()
            .iter()
            .map(|(_, process)| ProcessInfo {
                name: process.name().to_string_lossy().to_string(),
                pid: process.pid().as_u32(),
                memory: process.memory(),
            })
            .collect()
    }

    /// Retrieves system uptime.
    pub fn uptime(&self) -> Uptime {
        Uptime::from_seconds(sysinfo::System::uptime())
    }

    /// Retrieves system details such as OS name, version, kernel, hostname, and architecture.
    pub fn system_details(&self) -> (String, String, String, String, String) {
        (
            sysinfo::System::name().unwrap_or_else(|| "<unknown>".to_string()),
            sysinfo::System::os_version().unwrap_or_else(|| "<unknown>".to_string()),
            sysinfo::System::kernel_version().unwrap_or_else(|| "<unknown>".to_string()),
            sysinfo::System::host_name().unwrap_or_else(|| "<unknown>".to_string()),
            sysinfo::System::cpu_arch(),
        )
    }
}