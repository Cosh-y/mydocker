use crate::RmCommand;
use crate::container::{delete_metainfo, is_running};
use crate::run::ROOTFS_BASE_PATH;

pub fn rm(command: RmCommand) {
    let container_id = command.container_id.clone();
    // 检查容器是否已经在运行
    if is_running(&container_id) {
        println!("Container {} is still running", container_id);
        return;
    }

    // 删除容器的工作目录
    let container_path = format!("{}{}/", ROOTFS_BASE_PATH, container_id);
    std::fs::remove_dir_all(&container_path).expect("Failed to remove container directory");

    // 删除容器元信息
    delete_metainfo(&container_id);
}