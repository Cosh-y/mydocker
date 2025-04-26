use crate::network::*;

// 应该在某个文件里持久化的保存已创建的网络的配置信息，用网络名进行索引
pub const NETWORK_BASEPATH: &str = "/root/.mydocker/network/network/";

pub struct Network {
    name: String,    // 网络名
    subnet: SubNet,  // 子网
    driver: String,  // 网络类型（驱动类型）名
}

impl Network {
    pub fn new(name: &str, subnet: SubNet, driver: &str) -> Self {
        Network {
            name: name.to_string(),
            subnet,
            driver: driver.to_string(),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    // 将网络信息持久化到文件中
    pub fn dump(&self) {
        
    }
}