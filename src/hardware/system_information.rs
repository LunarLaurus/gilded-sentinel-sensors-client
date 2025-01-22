use sysinfo::{Components, Disks, Networks, System, Users};

use crate::data::models::{CpuInfo, DiskInfo, MemoryInfo, NetworkInfo, ProcessInfo, Uptime};

pub struct SystemInfo {
    system: System,
    networks: Networks,
    disks: Disks,
    components: Components,
    users: Users,
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

    /// Refreshes only the system-related data.
    pub fn refresh_system(&mut self) {
        self.system.refresh_all();
    }

    /// Refreshes only the network-related data.
    pub fn refresh_networks(&mut self) {
        self.networks.refresh(false);
    }

    /// Refreshes only the disk-related data.
    pub fn refresh_disks(&mut self) {
        self.disks.refresh(false);
    }

    /// Refreshes only the components data.
    pub fn refresh_components(&mut self) {
        self.components.refresh(false);
    }

    /// Refreshes only the users data.
    pub fn refresh_users(&mut self) {
        self.users.refresh();
    }

    /// Refreshes all system data.
    pub fn refresh_all(&mut self) {
        self.refresh_system();
        self.refresh_networks();
        self.refresh_disks();
        self.refresh_components();
        self.refresh_users();
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

    /// Retrieves system details such as OS name, version, kernel, hostname.
    pub fn system_details(&self) -> (String, String, String, String) {
        (
            sysinfo::System::name().unwrap_or_else(|| "<unknown>".to_string()),
            sysinfo::System::os_version().unwrap_or_else(|| "<unknown>".to_string()),
            sysinfo::System::kernel_version().unwrap_or_else(|| "<unknown>".to_string()),
            sysinfo::System::host_name().unwrap_or_else(|| "<unknown>".to_string()),
        )
    }
}
