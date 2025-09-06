use crate::network::*;
use crate::CreateNetworkCommand;

pub fn create_network(cmd: CreateNetworkCommand) {
    // 这里只做解析参数 + 调用驱动的工作
    let drivers = crate::network::DRIVERS.lock().unwrap();
    let driver = drivers.get(&cmd.driver).unwrap();
    let subnet = cmd.subnet.parse().unwrap();
    let network = driver.create(subnet, &cmd.name);
    
    // 将网络信息持久化到文件中
    register_network(network);

    log::info!("Created network: {}", cmd.name);
}

pub fn list_network() {
    
}
