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
    pub cores: Vec<CpuCoreData>,
}

#[derive(Serialize, Debug)]
pub struct SensorData {
    pub cpu_packages: Vec<CpuPackageData>,
    pub other_sensors: Vec<String>, // For sensors we don't specifically parse
}
