use crate::container::METAINFO_BASE_PATH;
use crate::run::ROOTFS_BASE_PATH;

pub fn prune() {
    let metainfo_dir = std::fs::read_dir(METAINFO_BASE_PATH).expect("Failed to read metainfo directory");
    for entry in metainfo_dir {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();
        if path.is_dir() {
            let config_file = path.join("config.json");
            let metainfo_content = std::fs::read_to_string(&config_file).expect("Failed to read config file");
            let metainfo: serde_json::Value = serde_json::from_str(&metainfo_content).expect("Failed to deserialize metainfo");
            let status = metainfo["status"].as_str().unwrap_or("");
            if status == "exited" {
                let container_id = path.file_name().unwrap().to_str().unwrap();

                let overlayfs_path = format!("{}{}/", ROOTFS_BASE_PATH, container_id);
                std::fs::remove_dir_all(&overlayfs_path).expect("Failed to remove container directory");

                let container_path = format!("{}{}/", METAINFO_BASE_PATH, container_id);
                std::fs::remove_dir_all(&container_path).expect("Failed to remove container directory");
                println!("Removed container {}", container_id);
            }
        }
    }
}