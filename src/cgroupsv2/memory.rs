use super::manager::{CGroupIf, ResourceConfig, CGROUP_ROOTPATH};


pub struct CGroupMemory {

}

impl CGroupMemory {
    pub fn new() -> Self {
        CGroupMemory {}
    }
}

impl CGroupIf for CGroupMemory {
    fn set(&self, path: &String, resource_config: &ResourceConfig) {
        let memory_limit = if let Some(memory) = &resource_config.memory {
            memory
        } else {
            return;
        };
        let memory_path = format!("{}/{}/memory.max", CGROUP_ROOTPATH, path);
        // memory_limit 是一个字符串，如 10G 10m 10M 20k 等
        let memory = if memory_limit.ends_with(&['g', 'G']) {
            memory_limit.trim_end_matches(&['g', 'G']).parse::<u64>().unwrap() * 1024 * 1024 * 1024
        } else if memory_limit.ends_with(&['m', 'M']) {
            memory_limit.trim_end_matches(&['m', 'M']).parse::<u64>().unwrap() * 1024 * 1024
        } else if memory_limit.ends_with(&['k', 'K']) {
            memory_limit.trim_end_matches(&['k', 'K']).parse::<u64>().unwrap() * 1024
        } else {
            panic!("Invalid memory limit format");
        };
        let memory_limit_str = format!("{}", memory);
        std::fs::write(&memory_path, memory_limit_str).expect("Failed to set memory limit");
    }
}