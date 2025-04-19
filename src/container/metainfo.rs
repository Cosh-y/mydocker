use serde::Serialize;
use log::info;
use rand::prelude::*;

use crate::RunCommand;

const METAINFO_BASE_PATH: &str = "/root/.mydocker/containers/";

#[derive(Serialize)]
struct Metainfo {
    pid: u32,
    id: String,
    command: RunCommand,
    status: String,
}

pub fn init_metainfo(pid: u32, command: RunCommand) {
    let metainfo = Metainfo {
        pid,
        id: gen_id(),
        command,
        status: "running".to_string(),
    };
    let metainfo_dir = format!("{}{}/", METAINFO_BASE_PATH, metainfo.id);
    std::fs::create_dir_all(&metainfo_dir).expect("Failed to create metainfo directory");
    let metainfo_file = metainfo_dir + "config.json";
    let metainfo_json = serde_json::to_string(&metainfo).expect("Failed to serialize metainfo");
    std::fs::write(&metainfo_file, metainfo_json).expect("Failed to write metainfo file");
    info!("Metainfo file created at {}", metainfo_file);
}

fn gen_id() -> String {
    let mut rng = rand::rng();
    let nums: Vec<i32> = (0..10).collect();
    let mut id = String::new();
    for _ in 0..10 {
        id.push_str(&nums.choose(&mut rng).unwrap().to_string());
    }
    id
}
