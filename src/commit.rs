use std::process::Command;

use crate::run::ROOTFS;

pub fn commit_container(image: &str) {
    let rootfs = ROOTFS.to_string() + "/merged";
    let target = ROOTFS.to_string() + "/" + image + ".tar";
    Command::new("tar")
        .args(&["-cf", &target, &rootfs])
        .status()
        .expect("failed to execute tar command");
}
