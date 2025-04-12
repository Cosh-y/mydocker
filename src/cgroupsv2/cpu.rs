use super::manager::{CGroupIf, ResourceConfig, CGROUP_ROOTPATH};

pub struct CGroupCPU {

}

impl CGroupCPU {
    pub fn new() -> Self {
        CGroupCPU {}
    }
}

impl CGroupIf for CGroupCPU {
    fn set(&self, path: &String, resource_config: &ResourceConfig) {
        let cpu_limit = if let Some(cpu) = resource_config.cpu {
            cpu
        } else {
            return;
        };
        let cpu_path = format!("{}/{}/cpu.max", CGROUP_ROOTPATH, path);
        let cpu_limit_str = format!("{} 100000", cpu_limit * 1000);
        std::fs::write(&cpu_path, cpu_limit_str).expect("Failed to set CPU limit");
    }
}