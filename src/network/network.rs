use std::collections::BTreeMap;
use std::sync::Mutex;
use std::net::IpAddr;
use futures::stream::TryStreamExt;
use rtnetlink::{LinkUnspec, new_connection};
use serde::{Serialize, Deserialize};
use std::io::Write;

use crate::{container, network::*};
use crate::exec::enter_container_netns;
use log::{error, info};

// 应该在某个文件里持久化的保存已创建的网络的配置信息，用网络名进行索引
// TODO：这个 Mutex 似乎不能避免多进程间的数据竞争，只是在单个进程下有效
static NETWORKS: Mutex<BTreeMap<String, Network>> = Mutex::new(BTreeMap::new());
pub const NETWORK_FILE: &str = "/root/.mydocker/network/network/network.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct Network {
    name: String,    // 网络名
    driver: String,  // 网络类型（驱动类型）名
}

impl Network {
    pub fn new(name: &str, driver: &str) -> Self {
        Network {
            name: name.to_string(),
            driver: driver.to_string(),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

pub fn dump_network() {
    let networks = NETWORKS.lock().unwrap();
    let mut file = std::fs::File::create(NETWORK_FILE).unwrap();
    serde_json::to_writer(&mut file, &*networks).unwrap();
}

pub fn load_network() -> std::io::Result<()> {
    let file = match std::fs::File::open(NETWORK_FILE) {
        Ok(f) => f,
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
            // 文件不存在则创建一个空文件并写入空的 JSON 对象
            std::fs::create_dir_all(std::path::Path::new(NETWORK_FILE).parent().unwrap())?;
            let mut f = std::fs::File::create(NETWORK_FILE)?;
            f.write_all(b"{}")?;
            std::fs::File::open(NETWORK_FILE)?
        }
        Err(e) => return Err(e),
    };
    let reader = std::io::BufReader::new(file);
    let networks: BTreeMap<String, Network> = serde_json::from_reader(reader)?;
    *NETWORKS.lock().unwrap() = networks;
    Ok(())
}

pub fn register_network(network: Network) {
    load_network().unwrap();
    let mut networks = NETWORKS.lock().unwrap();
    networks.insert(network.name.clone(), network);
    drop(networks);
    dump_network();
}

// 在 run 启动容器时，将容器 connect 到指定的网络
pub fn connect(network_name: &str, container_id: &str) {
    load_network().unwrap();
    let networks = NETWORKS.lock().unwrap();
    let drivers = DRIVERS.lock().unwrap();

    let network = networks.get(network_name).unwrap();
    let driver = drivers.get(&network.driver).unwrap();
    
    let ip = allocate_ip(network_name).unwrap();

    let mut ep = Endpoint {
        id: container_id.to_string(),
        network_name: network_name.to_string(),
        ip: ip,
        peer_name: None,
    };

    log::info!("In network connect");
    driver.connect(network_name, &mut ep);

    // 到容器的网络命名空间中配置 Endpoint
    config_endpoint_ip_address_and_route(container_id, &ep);
}

fn config_endpoint_ip_address_and_route(container_id: &str, ep: &Endpoint) {
    let container_pid = container::get_pid(container_id);
    
    TOKIO.block_on(async {
        // ip link set veth1 netns ns1
        HANDLE
            .link()
            .set(
                LinkUnspec::new_with_name(ep.peer_name.as_ref().unwrap())
                    .setns_by_pid(container_pid)
                    .build(),
            )
            .execute()
            .await
            .unwrap();
    });
    
    enter_container_netns(container_id);

    let tokio_runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let (connection, handle, _) = tokio_runtime.block_on(async { 
        new_connection().unwrap()
    });
    
    tokio_runtime.spawn(connection);

    tokio_runtime.block_on(async {
        // ip netns exec ns1 ip addr add 172.18.0.2/24 dev veth1
        let mut links = handle
            .link()
            .get()
            .match_name(ep.peer_name.as_ref().unwrap().to_string())
            .execute();
        let link = links.try_next().await.unwrap().unwrap();
        handle
            .address()
            .add(
                link.header.index,
                IpAddr::V4(ep.ip.ip()),
                ep.ip.prefix(),
            )
            .execute()
            .await
            .unwrap();

        // ip netns exec ns1 ip link set veth1 up
        handle
            .link()
            .set(LinkUnspec::new_with_name(ep.peer_name.as_ref().unwrap()).up().build())
            .execute()
            .await
            .unwrap();
    })
}
