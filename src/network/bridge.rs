use std::fmt::format;
use futures::stream::TryStreamExt;

// Bridge 是一种 NetworkDriver，用于创建基于 Bridge 的 Network
use crate::network::*;
use rtnetlink::{LinkBridge, LinkVeth, LinkUnspec};
use log::error;

pub struct Bridge {

}

impl NetworkDriver for Bridge {
    fn create(&self, subnet: SubNet, name: &str) -> Network {
        // 创建网络
        let network = Network::new(name, "bridge");
        // 记录网络的网段信息
        register_network_subnet(name, subnet);

        // 为网络创建网桥（交换机 + 路由器）
        self.init_bridge(&network);
        network
    }

    fn delete(&self) {
        // 删除网络
    }

    // 创建一对 Veth 设备，将其中一个添加到桥设备上
    fn connect(&self, network_name: &str, ep: &mut Endpoint) {
        // 连接网络
        log::info!("here.");
        let veth_name = format!("veth{}", &ep.id[0..5]);
        let veth_peer_name = format!("veth{}peer", &ep.id[0..5]);
        ep.peer_name = Some(veth_peer_name.clone());
        TOKIO.block_on(async {
            // ip link add veth0 type veth peer name veth1
            HANDLE
                .link()
                .add(LinkVeth::new(&veth_name, &veth_peer_name).build())
                .execute()
                .await
                .unwrap();
            log::info!("Created veth pair: {} and {}", veth_name, veth_peer_name);

            let mut bridges = HANDLE.link().get().match_name(network_name.to_string()).execute();

            let bridge = bridges.try_next().await.unwrap().unwrap().header.index;

            // ip link set veth0 master bridge
            HANDLE
                .link()
                .set(
                    LinkUnspec::new_with_name(&veth_name)
                        .controller(bridge)
                        .build()
                )
                .execute()
                .await
                .unwrap();

            // ip link set veth0 up
            HANDLE
                .link()
                .set(LinkUnspec::new_with_name(&veth_name).up().build())
                .execute()
                .await
                .unwrap_or_else(|e| { error!("Failed to set veth up: {}", e) });
        })
    }

    fn disconnect(&self, endpoint_id: &str) {
        // 断开网络
    }
}

impl Bridge {
    fn init_bridge(&self, network: &Network) {
        TOKIO.block_on(async {
            // ip link add name <bridge_name> type bridge
            HANDLE
                .link()
                .add(LinkBridge::new(network.get_name()).build())
                .execute()
                .await
                .unwrap_or_else(|e| { error!("Failed to create bridge: {}", e) });

            // ip link set <bridge_name> up
            HANDLE
                .link()
                .set(LinkUnspec::new_with_name(network.get_name()).up().build())
                .execute()
                .await
                .unwrap_or_else(|e| { error!("Failed to set bridge up: {}", e) });
        });
    }

    fn set_bridge_ip(&self, network: &Network) {
        
    }

    fn set_bridge_up(&self, network: &Network) {
        
    }
}
