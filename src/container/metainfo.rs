use serde::{Serialize, Deserialize};
use log::info;
use rand::prelude::*;

use crate::RunCommand;
use crate::PsCommand;

pub const METAINFO_BASE_PATH: &str = "/root/.mydocker/containers/";

#[derive(Serialize, Deserialize)]
struct Metainfo {
    pid: Option<u32>,
    id: String,
    command: RunCommand,
    status: String,
}

pub fn init_metainfo(container_id: &str, pid: u32, command: RunCommand) -> String {
    let metainfo = Metainfo {
        pid: Some(pid),
        id: container_id.to_string(),
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

pub fn delete_metainfo(container_id: &str) {
    let metainfo_dir = format!("{}{}/", METAINFO_BASE_PATH, container_id);
    std::fs::remove_dir_all(&metainfo_dir).expect("Failed to delete metainfo directory");
}

pub fn gen_id() -> String {
    let mut rng = rand::rng();
    let nums: Vec<i32> = (0..10).collect();
    let mut id = String::new();
    for _ in 0..10 {
        id.push_str(&nums.choose(&mut rng).unwrap().to_string());
    }
    id
}

pub fn record_exit(container_id: &str) {
    let metainfo_file = format!("{}{}/config.json", METAINFO_BASE_PATH, container_id);

    // use serde_json to read metainfo file
    let metainfo_content = std::fs::read_to_string(&metainfo_file).expect("Failed to read metainfo file");
    let mut metainfo: Metainfo = serde_json::from_str(&metainfo_content).expect("Failed to deserialize metainfo");

    metainfo.status = "exited".to_string();
    metainfo.pid = None;

    let metainfo_json = serde_json::to_string(&metainfo).expect("Failed to serialize metainfo");
    std::fs::write(&metainfo_file, metainfo_json).expect("Failed to write metainfo file");
}

pub fn record_running(container_id: &str, pid: u32) {
    let metainfo_file = format!("{}{}/config.json", METAINFO_BASE_PATH, container_id);

    // use serde_json to read metainfo file
    let metainfo_content = std::fs::read_to_string(&metainfo_file).expect("Failed to read metainfo file");
    let mut metainfo: Metainfo = serde_json::from_str(&metainfo_content).expect("Failed to deserialize metainfo");

    metainfo.status = "running".to_string();
    metainfo.pid = Some(pid);

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

fn get_metainfo(container_id: &str) -> Metainfo {
    let metainfo_file = format!("{}{}/config.json", METAINFO_BASE_PATH, container_id);
    let metainfo_content = std::fs::read_to_string(&metainfo_file).expect("Failed to read metainfo file");
    let metainfo: Metainfo = serde_json::from_str(&metainfo_content).expect("Failed to deserialize metainfo");
    return metainfo;
}

pub fn get_pid(container_id: &str) -> u32 {
    get_metainfo(container_id).pid.unwrap()
}

pub fn is_running(container_id: &str) -> bool {
    get_metainfo(container_id).status == "running"
}

pub fn get_volume(container_id: &str) -> Option<String> {
    get_metainfo(container_id).command.volume
}

pub fn get_command(container_id: &str) -> RunCommand {
    get_metainfo(container_id).command
}

pub fn metainfo_exists(container_id: &str) -> bool {
    let metainfo_file = format!("{}{}/config.json", METAINFO_BASE_PATH, container_id);
    std::fs::metadata(metainfo_file).is_ok()
}
