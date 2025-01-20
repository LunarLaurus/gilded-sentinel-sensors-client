use serde::Serialize;

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
pub struct SensorData {
    pub cpu_packages: Vec<CpuPackageData>,
}
