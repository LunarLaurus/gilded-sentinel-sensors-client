use crate::{
    data::models::{ComponentInfo, CpuInfo, DiskInfo, MemoryInfo, NetworkInfo, ProcessInfo, Uptime},
    hardware::system_information::SystemInfo,
};
use log::info;
use sysinfo::{Components, Users};

pub struct SysInfoMonitor {
    system_info: SystemInfo,
}

#[allow(dead_code)] // Suppress warnings for unused functions.
impl SysInfoMonitor {
    /// Creates a new instance of `SysInfoMonitor`.
    pub fn new() -> Self {
        Self {
            system_info: SystemInfo::new(),
        }
    }

    /// Refreshes all system data.
    pub fn refresh_all(&mut self) {
        self.system_info.refresh_all();
    }

    /// Refreshes system-related data.
    pub fn refresh_system(&mut self) {
        self.system_info.refresh_system();
    }

    /// Refreshes components-related data.
    pub fn refresh_components(&mut self) {
        self.system_info.refresh_components();
    }

    /// Refreshes disk-related data.
    pub fn refresh_disks(&mut self) {
        self.system_info.refresh_disks();
    }

    /// Refreshes network-related data.
    pub fn refresh_networks(&mut self) {
        self.system_info.refresh_networks();
    }

    /// Refreshes user-related data.
    pub fn refresh_users(&mut self) {
        self.system_info.refresh_users();
    }

    /// Returns memory information.
    pub fn get_memory_info(&mut self) -> MemoryInfo {
        self.refresh_system();
        self.system_info.memory_info()
    }

    /// Logs memory information.
    pub fn log_memory_info(&mut self) {
        let memory_info = self.get_memory_info();
        info!("Total Memory: {} bytes", memory_info.total);
        info!("Used Memory: {} bytes", memory_info.used);
        info!("Total Swap: {} bytes", memory_info.total_swap);
        info!("Used Swap: {} bytes", memory_info.used_swap);
    }

    /// Returns CPU information.
    pub fn get_cpu_info(&mut self) -> CpuInfo {
        self.refresh_system();
        self.system_info.cpu_info()
    }

    /// Logs CPU usage information.
    pub fn log_cpu_info(&mut self) {
        let cpu_info = self.get_cpu_info();
        info!("CPU Core Count: {}", cpu_info.core_count);
        info!("CPU Architecture: {}", cpu_info.cpu_arch);
        info!("CPU Usage per Core:");
        for (i, usage) in cpu_info.usage_per_core.iter().enumerate() {
            info!("Core {}: {:.2}%", i, usage);
        }
    }

    /// Returns user information.
    pub fn get_user_info(&mut self) -> &Users {
        self.refresh_users();
        self.system_info.get_users()
    }

    /// Logs user information.
    pub fn log_user_info(&mut self) {
        let users = self.get_user_info();
        info!("Users:");
        for user in users.iter() {
            info!("Name: {}", user.name());
        }
    }

    /// Returns disk usage information.
    pub fn get_disk_info(&mut self) -> Vec<DiskInfo> {
        self.refresh_disks();
        self.system_info.disk_info()
    }

    /// Logs disk usage information.
    pub fn log_disk_info(&mut self) {
        let disk_info = self.get_disk_info();
        info!("Disk Usage:");
        for disk in disk_info {
            info!(
                "Disk: {} | Total: {} bytes | Available: {} bytes | Read: {} bytes | Written: {} bytes",
                disk.name, disk.total_space, disk.available_space, disk.read_bytes, disk.written_bytes
            );
        }
    }

    /// Returns network usage information.
    pub fn get_network_info(&mut self) -> Vec<NetworkInfo> {
        self.refresh_networks();
        self.system_info.network_info()
    }

    /// Logs network usage information.
    pub fn log_network_info(&mut self) {
        let network_info = self.get_network_info();
        info!("Network Usage:");
        for network in network_info {
            info!(
                "Interface: {} | Received: {} bytes | Transmitted: {} bytes | MTU: {:?}",
                network.interface_name, network.received, network.transmitted, network.mtu
            );
        }
    }

    /// Returns process list information.
    pub fn get_process_info(&mut self) -> Vec<ProcessInfo> {
        self.refresh_system();
        self.system_info.process_info()
    }

    /// Logs process list information.
    pub fn log_process_info(&mut self) {
        let process_info = self.get_process_info();
        info!("Process List:");
        for process in process_info {
            info!(
                "Process: {} | PID: {} | Memory: {} bytes",
                process.name, process.pid, process.memory
            );
        }
    }

    /// Returns system details.
    pub fn get_system_details(&mut self) -> (String, String, String, String) {
        self.refresh_system();
        self.system_info.system_details()
    }

    /// Logs system details.
    pub fn log_system_details(&mut self) {
        let (os_name, os_version, kernel_version, hostname) = self.get_system_details();
        info!("OS Name: {}", os_name);
        info!("OS Version: {}", os_version);
        info!("Kernel Version: {}", kernel_version);
        info!("Hostname: {}", hostname);
    }

    /// Returns system uptime.
    pub fn get_uptime(&mut self) -> Uptime {
        self.refresh_system();
        self.system_info.uptime()
    }

    /// Logs system uptime.
    pub fn log_uptime(&mut self) {
        let uptime = self.get_uptime();
        info!("System Uptime: {}", uptime.to_string());
    }

    /// Returns system components as a read-only reference.
    pub fn get_components(&mut self) -> &Components {
        self.refresh_components();
        self.system_info.get_components()
    }

    /// Returns a vector of `ComponentInfo` DTOs representing system components.
    pub fn get_components_info(&mut self) -> Vec<ComponentInfo> {
        self.refresh_components();
        self.get_components()
            .iter()
            .map(ComponentInfo::from)
            .collect()
    }

    /// Logs detailed information about all system components.
    pub fn log_components(&mut self) {
        let components_info = self.get_components_info();
        info!("System Components:");
        for component in components_info {
            info!(
                "Component: {} | Temperature: {} | Max Temperature: {} | Critical Temperature: {}",
                component.label,
                component
                    .temperature
                    .map(|t| format!("{:.2}°C", t))
                    .unwrap_or_else(|| "Unavailable".to_string()),
                component
                    .max_temperature
                    .map(|t| format!("{:.2}°C", t))
                    .unwrap_or_else(|| "Unavailable".to_string()),
                component
                    .critical_temperature
                    .map(|t| format!("{:.2}°C", t))
                    .unwrap_or_else(|| "Unavailable".to_string()),
            );
        }
    }

    /// Logs essential system information by invoking all log methods.
    pub fn setup_monitoring(&mut self) {
        info!("Setting up system monitoring...");
        self.log_system_details();
        self.log_uptime();
        self.log_memory_info();
        self.log_cpu_info();
        self.log_components();
        info!("System monitoring setup complete.");
    }
}
