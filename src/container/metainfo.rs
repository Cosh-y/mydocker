use serde::{Serialize, Deserialize};
use log::info;
use rand::prelude::*;

use crate::RunCommand;
use crate::PsCommand;

const METAINFO_BASE_PATH: &str = "/root/.mydocker/containers/";

#[derive(Serialize, Deserialize)]
struct Metainfo {
    pid: u32,
    id: String,
    command: RunCommand,
    status: String,
}

pub fn init_metainfo(pid: u32, command: RunCommand) -> String {
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
    return metainfo.id;
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

pub fn record_exit(container_id: String) {
    let metainfo_file = format!("{}{}/config.json", METAINFO_BASE_PATH, container_id);

    // use serde_json to read metainfo file
    let metainfo_content = std::fs::read_to_string(&metainfo_file).expect("Failed to read metainfo file");
    let mut metainfo: Metainfo = serde_json::from_str(&metainfo_content).expect("Failed to deserialize metainfo");

    metainfo.status = "exited".to_string();
    
    let metainfo_json = serde_json::to_string(&metainfo).expect("Failed to serialize metainfo");
    std::fs::write(&metainfo_file, metainfo_json).expect("Failed to write metainfo file");
}

pub fn ps(command: PsCommand) {
    if command.all {
        // TODO: Implement logic to show all containers
    }
    let metainfo_dir = std::fs::read_dir(METAINFO_BASE_PATH).expect("Failed to read metainfo directory");
    for entry in metainfo_dir {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();
        if path.is_dir() {
            let config_file = path.join("config.json");
            if config_file.exists() {
                let config_content = std::fs::read_to_string(config_file).expect("Failed to read config file");
                println!("{}", config_content);
            }
        }
    }
}
