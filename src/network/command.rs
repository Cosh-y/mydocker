use crate::network::*;
use crate::CreateNetworkCommand;

pub fn create_network(cmd: CreateNetworkCommand) -> Network {
    let drivers = crate::network::DRIVERS.lock().unwrap();
    let driver = drivers.get(&cmd.driver).unwrap();
    let subnet = cmd.subnet.parse().unwrap();
    let network = driver.create(subnet, &cmd.name);
    network.dump();
    network
}

pub fn list_network() {
    
}
