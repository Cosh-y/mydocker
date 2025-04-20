use std::process::Command;

use crate::run::{ROOTFS_BASE_PATH, IMAGE_BASE_PATH};

pub fn commit_container(container_id: &str, image: &str) {
    let rootfs = ROOTFS_BASE_PATH.to_string() + container_id + "/merged";
    let target = IMAGE_BASE_PATH.to_string() + image + ".tar";
    Command::new("tar")
        .args(&["-cf", &target, &rootfs])
        .status()
        .expect("failed to execute tar command");
}
