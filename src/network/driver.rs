use std::collections::BTreeMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use rtnetlink::{new_connection, Handle};
use tokio::runtime::Runtime;

// 本文件定义 NetworkDriver 的接口
// 本项目先实现 Bridge 类型的 NetworkDriver
// 原生 docker 具有的 NetworkDriver 类型还有 overlay 和 macvlan
use crate::network::*;

// 用一个全局变量记录可选的 driver，当用户创建网络时，选用用户指定的 driver
pub static DRIVERS: Mutex<BTreeMap<String, Box<dyn NetworkDriver>>> = Mutex::new(BTreeMap::new());

// 全局 runtime
pub static TOKIO: Lazy<Runtime> = Lazy::new(|| {
    Runtime::new().unwrap()
});

// 全局 handle
pub static HANDLE: Lazy<Handle> = Lazy::new(|| {
    let (connection, handle, _) = new_connection().unwrap();
    TOKIO.spawn(connection); // 注意这里用全局 runtime
    handle
});

pub trait NetworkDriver: Send + Sync {
    fn create(&self, subnet: SubNet, name: &str) -> Network;
    fn delete(&self);
    fn connect(&self, bridge: &Network, veth: &Endpoint);
    fn disconnect(&self, bridge: &Network, veth: &Endpoint);   
}

pub fn register_driver(name: &str, driver: Box<dyn NetworkDriver>) {
    let mut drivers = DRIVERS.lock().unwrap();
    drivers.insert(name.to_string(), driver);
}
