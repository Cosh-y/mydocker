use ipnetwork::Ipv4Network;

pub struct Endpoint {
    pub id: String,
    pub network_name: String,
    pub ip: Ipv4Network,
    pub peer_name: Option<String>,  // 在容器内部的 Veth-peer 的名称
}