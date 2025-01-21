use crate::hardware::sysinfo::{CpuInfo, DiskInfo, MemoryInfo, NetworkInfo, ProcessInfo, SystemInfo};
use log::info;

pub struct SysInfoMonitor {
    system_info: SystemInfo,
}

impl SysInfoMonitor {
    /// Creates a new instance of `SysInfoMonitor`.
    pub fn new() -> Self {
        Self {
            system_info: SystemInfo::new(),
        }
    }

    /// Refreshes the system data.
    fn refresh(&mut self) {
        self.system_info.refresh();
    }

    /// Returns memory information as a DTO.
    pub fn get_memory_info(&mut self) -> MemoryInfo {
        self.refresh();
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

    /// Returns CPU information as a DTO.
    pub fn get_cpu_info(&mut self) -> CpuInfo {
        self.refresh();
        self.system_info.cpu_info()
    }

    /// Logs CPU usage information.
    pub fn log_cpu_info(&mut self) {
        let cpu_info = self.get_cpu_info();
        info!("CPU Core Count: {}", cpu_info.core_count);
        info!("CPU Usage per Core:");
        for (i, usage) in cpu_info.usage_per_core.iter().enumerate() {
            info!("Core {}: {:.2}%", i, usage);
        }
    }

    /// Returns disk usage information as a list of DTOs.
    pub fn get_disk_info(&mut self) -> Vec<DiskInfo> {
        self.refresh();
        self.system_info.disk_info()
    }

    /// Logs disk usage information.
    pub fn log_disk_info(&mut self) {
        let disk_info = self.get_disk_info();
        info!("Disk Usage:");
        for disk in disk_info {
            info!(
                "Disk: {} | Total: {} bytes | Available: {} bytes",
                disk.name, disk.total_space, disk.available_space
            );
        }
    }

    /// Returns network usage information as a list of DTOs.
    pub fn get_network_info(&mut self) -> Vec<NetworkInfo> {
        self.refresh();
        self.system_info.network_info()
    }

    /// Logs network usage information.
    pub fn log_network_info(&mut self) {
        let network_info = self.get_network_info();
        info!("Network Usage:");
        for network in network_info {
            info!(
                "Interface: {} | Received: {} bytes | Transmitted: {} bytes",
                network.interface_name, network.received, network.transmitted
            );
        }
    }

    /// Returns process list information as a list of DTOs.
    #[allow(dead_code)] // Suppress warnings for unused function.
    pub fn get_process_info(&mut self) -> Vec<ProcessInfo> {
        self.refresh();
        self.system_info.process_info()
    }

    /// Logs process list information.
    #[allow(dead_code)] // Suppress warnings for unused function.
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

    /// Returns system details like OS name, version, kernel, and hostname.
    pub fn get_system_details(&mut self) -> (Option<String>, Option<String>, Option<String>, Option<String>) {
        self.refresh();
        self.system_info.system_details()
    }

    /// Logs system details.
    pub fn log_system_details(&mut self) {
        let (os_name, os_version, kernel_version, hostname) = self.get_system_details();
        info!("OS Name: {:?}", os_name);
        info!("OS Version: {:?}", os_version);
        info!("Kernel Version: {:?}", kernel_version);
        info!("Hostname: {:?}", hostname);
    }

    /// Returns system uptime.
    pub fn get_uptime(&mut self) -> u64 {
        self.refresh();
        self.system_info.uptime()
    }

    /// Logs system uptime.
    pub fn log_uptime(&mut self) {
        let uptime = self.get_uptime();
        info!("System Uptime: {} seconds", uptime);
    }

    /// Logs all system information by invoking all log methods.
    pub fn setup_monitoring(&mut self) {
        info!("Setting up system monitoring...");
        self.log_system_details();
        self.log_uptime();
        self.log_memory_info();
        self.log_cpu_info();
        self.log_disk_info();
        self.log_network_info();
        // self.log_process_info(); // Kept commented for potential future use
        info!("System monitoring setup complete.");
    }
}
