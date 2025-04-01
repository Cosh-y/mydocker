use log::warn;

use super::manager::{CGroupIf, ResourceConfig};


pub struct CGroupMemory {

}

impl CGroupMemory {
    pub fn new() -> Self {
        CGroupMemory {}
    }
}

impl CGroupIf for CGroupMemory {
    fn set(&self, path: &String, resource_config: &ResourceConfig) {
        warn!("Memory cgroup is not implemented yet");
    }
}