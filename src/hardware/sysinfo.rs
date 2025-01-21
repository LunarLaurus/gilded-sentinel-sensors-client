use std::fmt;

use sysinfo::{Components, Disks, Networks, Pid, Signal, System, User, Users};

pub struct MemoryInfo {
    pub total: u64,
    pub used: u64,
    pub total_swap: u64,
    pub used_swap: u64,
}

pub struct CpuInfo {
    pub usage_per_core: Vec<f32>,
    pub core_count: usize,
    pub cpu_arch: String, // Added CPU architecture
}

pub struct DiskInfo {
    pub name: String,
    pub total_space: u64,
    pub available_space: u64,
    pub read_bytes: u64,
    pub written_bytes: u64,
}

pub struct NetworkInfo {
    pub interface_name: String,
    pub received: u64,
    pub transmitted: u64,
    pub mtu: Option<u64>,
}

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

pub struct Uptime {
    pub days: u64,
    pub hours: u64,
    pub minutes: u64,
    pub seconds: u64,
    pub total_seconds: u64
}

impl Uptime {
    /// Creates a new `Uptime` object from total seconds.
    pub fn from_seconds(total_seconds: u64) -> Self {
        let mut remaining_seconds = total_seconds;
        let days = remaining_seconds / 86400;
        remaining_seconds -= days * 86400;
        let hours = remaining_seconds / 3600;
        remaining_seconds -= hours * 3600;
        let minutes = remaining_seconds / 60;
        let seconds = remaining_seconds % 60;

        Uptime {
            days,
            hours,
            minutes,
            seconds,
            total_seconds
        }
    }

    /// Formats the uptime as a human-readable string.
    pub fn to_string(&self) -> String {
        return format!(
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
        let mut system = System::new_all(); // Initialize with all data
        let networks = Networks::new_with_refreshed_list();
        let disks = Disks::new_with_refreshed_list();
        let components = Components::new_with_refreshed_list();
        let users = Users::new_with_refreshed_list();

        system.refresh_all(); // Refresh system data at initialization
        Self {
            system,
            networks,
            disks,
            components,
            users,
        }
    }

    /// Refreshes the system data.
    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }

    /// Gets memory information.
    pub fn memory_info(&self) -> MemoryInfo {
        MemoryInfo {
            total: self.system.total_memory(),
            used: self.system.used_memory(),
            total_swap: self.system.total_swap(),
            used_swap: self.system.used_swap(),
        }
    }

    /// Gets CPU information.
    pub fn cpu_info(&self) -> CpuInfo {
        CpuInfo {
            usage_per_core: self
                .system
                .cpus()
                .iter()
                .map(|cpu| cpu.cpu_usage())
                .collect(),
            core_count: self.system.cpus().len(),
            cpu_arch: sysinfo::System::cpu_arch(), // CPU architecture
        }
    }

    /// Gets disk usage information.
    pub fn disk_info(&self) -> Vec<DiskInfo> {
        self.disks
            .iter()
            .map(|disk| {
                let usage = disk.usage(); // Disk I/O statistics
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

    /// Gets network usage information.
    pub fn network_info(&self) -> Vec<NetworkInfo> {
        self.networks
            .iter()
            .map(|(name, data)| NetworkInfo {
                interface_name: name.clone(),
                received: data.received(),
                transmitted: data.transmitted(),
                mtu: Some(data.mtu()), // MTU information
            })
            .collect()
    }

    /// Gets a list of processes.
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

    /// Gets system uptime in seconds.
    pub fn uptime(&self) -> u64 {
        sysinfo::System::uptime()
    }

    /// Gets system details (OS name, version, kernel, hostname, and architecture).
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
