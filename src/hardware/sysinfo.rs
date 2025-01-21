use sysinfo::{CpuExt, DiskExt, NetworkExt, PidExt, ProcessExt, System, SystemExt};

pub struct MemoryInfo {
    pub total: u64,
    pub used: u64,
    pub total_swap: u64,
    pub used_swap: u64,
}

pub struct CpuInfo {
    pub usage_per_core: Vec<f32>,
    pub core_count: usize,
}

pub struct DiskInfo {
    pub name: String,
    pub total_space: u64,
    pub available_space: u64,
}

pub struct NetworkInfo {
    pub interface_name: String,
    pub received: u64,
    pub transmitted: u64,
}

pub struct ProcessInfo {
    pub name: String,
    pub pid: u32,
    pub memory: u64,
}

pub struct SystemInfo {
    system: System,
}

impl SystemInfo {
    /// Initializes a new `SystemInfo` instance and refreshes all data.
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        Self { system }
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
        }
    }

    /// Gets disk usage information.
    pub fn disk_info(&self) -> Vec<DiskInfo> {
        self.system
            .disks()
            .iter()
            .map(|disk| DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                total_space: disk.total_space(),
                available_space: disk.available_space(),
            })
            .collect()
    }

    /// Gets network usage information.
    pub fn network_info(&self) -> Vec<NetworkInfo> {
        self.system
            .networks()
            .into_iter()
            .map(|(name, data)| NetworkInfo {
                interface_name: name.clone(),
                received: data.received(),
                transmitted: data.transmitted(),
            })
            .collect()
    }

    /// Gets a list of processes.
    pub fn process_info(&self) -> Vec<ProcessInfo> {
        self.system
            .processes()
            .iter()
            .map(|(pid, process)| ProcessInfo {
                name: process.name().to_string(),
                pid: pid.as_u32(),
                memory: process.memory(),
            })
            .collect()
    }

    /// Gets system uptime in seconds.
    pub fn uptime(&self) -> u64 {
        self.system.uptime()
    }

    /// Gets system details (OS name, version, kernel, and hostname).
    pub fn system_details(
        &self,
    ) -> (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ) {
        (
            self.system.name(),
            self.system.os_version(),
            self.system.kernel_version(),
            self.system.host_name(),
        )
    }
}
