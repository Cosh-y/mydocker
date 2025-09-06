use std::net::Ipv4Addr;
use std::{collections::BTreeMap, str::FromStr};
use std::sync::Mutex;
use std::io::Write;
use ipnetwork::Ipv4Network;
use serde::{Deserialize, Serialize};

use log::info;

#[derive(Serialize, Deserialize, Debug)]
pub struct SubNet {
    ip: [u8; 4], // 子网地址
    mask: u8,    // 子网掩码，比如 24
    bitmap: Vec<u8>, // 用于分配空闲 IP, 每一位表示一个 IP 地址的使用状态
}

impl FromStr for SubNet {
    type Err = std::io::Error;

    // from strings like 192.168.0.0/24
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid subnet format",
            ));
        }

        let ip_parts: Vec<&str> = parts[0].split('.').collect();
        if ip_parts.len() != 4 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid IP address format",
            ));
        }

        let mut ip = [0; 4];
        for i in 0..4 {
            ip[i] = ip_parts[i].parse::<u8>().map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid IP address")
            })?;
        }

        let mask = parts[1]
            .parse::<u8>()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid mask"))?;

        let size = 1 << (32 - mask);
        let mut bitmap = vec![0; (size + 7) / 8];
        bitmap[0] = 0b00000001; // 标记第一个 IP 地址为已分配（通常是网络地址）

        Ok(SubNet { ip, mask, bitmap })
    }
}

// 保存网络名和网段信息的映射
static IPAM: Mutex<BTreeMap<String, SubNet>> = Mutex::new(BTreeMap::new());
const IPAM_FILE: &str = "/root/.mydocker/network/ipam/subnet.json";

fn load_ipam() -> std::io::Result<()> {
    let file = match std::fs::File::open(IPAM_FILE) {
        Ok(f) => f,
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
            // 文件不存在则创建一个空文件并写入空的 JSON 对象
            std::fs::create_dir_all(std::path::Path::new(IPAM_FILE).parent().unwrap())?;
            let mut f = std::fs::File::create(IPAM_FILE)?;
            f.write_all(b"{}")?;
            std::fs::File::open(IPAM_FILE)?
        }
        Err(e) => return Err(e),
    };
    let reader = std::io::BufReader::new(file);
    let ipam: BTreeMap<String, SubNet> = serde_json::from_reader(reader)?;
    *IPAM.lock().unwrap() = ipam;
    Ok(())
}

fn dump_ipam() -> std::io::Result<()> {
    let ipam = IPAM.lock().unwrap();
    let file = std::fs::File::create(IPAM_FILE)?;
    let writer = std::io::BufWriter::new(file);
    serde_json::to_writer(writer, &*ipam)?;
    Ok(())
}

pub fn allocate_ip(network: &str) -> Option<Ipv4Network> {
    load_ipam().unwrap();
    let mut ipam = IPAM.lock().unwrap();
    if let Some(subnet) = ipam.get_mut(network) {
        if let Some(ip) = subnet.allocate_ip() {
            let ipv4 = Ipv4Network::new(Ipv4Addr::from(ip), subnet.mask).unwrap();
            info!("Allocated IP: {} for network: {}", ipv4, network);
            drop(ipam);
            dump_ipam().unwrap();
            return Some(ipv4);
        }
    }
    None
}

pub fn release_ip(network: &str, ip: [u8; 4]) {
    load_ipam().unwrap();
    let mut ipam = IPAM.lock().unwrap();
    if let Some(subnet) = ipam.get_mut(network) {
        subnet.release_ip(ip);
        dump_ipam().unwrap();
    }
}

pub fn register_network_subnet(name: &str, subnet: SubNet) {
    load_ipam().unwrap();
    let mut ipam = IPAM.lock().unwrap();
    ipam.insert(name.to_string(), subnet);
    drop(ipam); // 释放锁
    dump_ipam().unwrap();
}

impl SubNet {
    fn allocate_ip(&mut self) -> Option<[u8; 4]> {
        let size = 1 << (32 - self.mask);
        for i in 0..size {
            let byte_index = i / 8;
            let bit_index = i % 8;
            if self.bitmap[byte_index] & (1 << bit_index) == 0 {
                self.bitmap[byte_index] |= 1 << bit_index;
                let ip = self.ip;
                let base = u32::from_be_bytes(ip);
                let allocated_ip = base + i as u32;
                let allocated_ip = allocated_ip.to_be_bytes();
                return Some(allocated_ip);
            }
        }
        None
    }

    fn release_ip(&mut self, ip: [u8; 4]) {
        let size = 1 << (32 - self.mask);
        let index = (u32::from_be_bytes(ip) - u32::from_be_bytes(self.ip)) % size;
        let byte_index = index / 8;
        let bit_index = index % 8;
        self.bitmap[byte_index as usize] &= !(1 << bit_index);
    }
}