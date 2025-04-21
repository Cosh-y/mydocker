use crate::container::METAINFO_BASE_PATH;

pub fn log(container_id: &str) {
    let log_path = format!("{}{}/container.log", METAINFO_BASE_PATH, container_id);
    let buffer = std::fs::read_to_string(log_path).expect("Failed to read log file");
    println!("{}", buffer);
}