// Bridge 是一种 NetworkDriver，用于创建基于 Bridge 的 Network
use crate::network::*;
use rtnetlink::LinkBridge;
use log::error;

pub struct Bridge {

}

impl NetworkDriver for Bridge {
    fn create(&self, subnet: SubNet, name: &str) -> Network {
        // 创建网络
        let network = Network::new(name, subnet, "bridge");
        self.init_bridge(&network);
        network
    }

    fn delete(&self) {
        // 删除网络
    }

    fn connect(&self, bridge: &Network, veth: &Endpoint) {
        // 连接网络
    }

    fn disconnect(&self, bridge: &Network, veth: &Endpoint) {
        // 断开网络
    }
}

impl Bridge {
    fn init_bridge(&self, network: &Network) {
        TOKIO.block_on(async {
            HANDLE
                .link()
                .add(LinkBridge::new(network.get_name()).build())
                .execute()
                .await
                .unwrap_or_else(|e| { error!("Failed to create bridge: {}", e) });


        });
    }

    fn set_bridge_ip(&self, network: &Network) {
        
    }

    fn set_bridge_up(&self, network: &Network) {
        
    }
}
