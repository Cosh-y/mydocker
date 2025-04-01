use super::cpu::CGroupCPU;
use super::memory::CGroupMemory;

pub const CGROUP_ROOTPATH: &str = "/sys/fs/cgroup";

pub trait CGroupIf {
    fn set(&self, path: &String, resource_config: &ResourceConfig);
}

pub struct ResourceConfig {
    pub cpu: Option<u32>,
    pub memory: Option<u32>,
}

pub struct CGroupManager {
    path: String,
    cgroups: Vec<Box<dyn CGroupIf>>,
}

impl CGroupManager {
    pub fn new(path: String) -> Self {
        CGroupManager {
            path,
            cgroups: vec![
                Box::new(CGroupCPU::new()),
                Box::new(CGroupMemory::new()),
            ],
        }
    }

    pub fn create_cgroup(&self) {
        // 创建 cgroup 目录
        let cgroup_path = format!("{}/{}", CGROUP_ROOTPATH, self.path);
        std::fs::create_dir_all(cgroup_path).expect("Failed to create cgroup directory");
    }

    pub fn destroy_cgroup(&self) {
        // 删除 cgroup 目录
        let cgroup_path = format!("{}/{}", CGROUP_ROOTPATH, self.path);
        std::fs::remove_dir_all(cgroup_path).expect("Failed to remove cgroup directory");
    }

    pub fn add_process(&self, pid: u32) {
        // 将进程添加到 cgroup
        let cgroup_path = format!("{}/{}", CGROUP_ROOTPATH, self.path);
        let pid_path = format!("{}/cgroup.procs", cgroup_path);
        std::fs::write(pid_path, pid.to_string()).expect("Failed to add process to cgroup");
    }

    pub fn set(&self, resource_config: ResourceConfig) {
        for cgroup in &self.cgroups {
            cgroup.set(&self.path, &resource_config);
        }
    }
}