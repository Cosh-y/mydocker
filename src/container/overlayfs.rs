use log::info;
use std::path::PathBuf;
use std::fs::*;
use std::process::Command;

use crate::run::{IMAGE_BASE_PATH, ROOTFS_BASE_PATH};

// 这里在确定 root 参数类型时从 String、&String 和 &str 中选择了 &str
// String 涉及所有权的转移
// &String 的引用不如 &str 灵活，比如 &str 能接收 "abc" 这样的字符串字面量，而 &String 不能
// &str 还能接收 String 的引用，会自动调用 deref 进行转换
pub fn new_workspace(container_id: &str, image: &str, volumn: Option<&str>) {
    info!("Creating overlayfs workspace at {}{}", ROOTFS_BASE_PATH, container_id);
    info!("Create some directories and mount overlayfs to merged.");
    create_lower(container_id, image);
    create_others(container_id);
    mount_overlayfs(container_id, image);

    if let Some(volumn) = volumn {
        info!("Mounting volume {}", volumn);
        let (volume, mount_point) = parse_volume(volumn);
        let mount_point = PathBuf::from(format!("{}{}/merged{}", ROOTFS_BASE_PATH, container_id, mount_point.display()));
        if !mount_point.exists() {
            panic!("Mount point {} does not exist", mount_point.display());
        }
        if !mount_point.is_dir() {
            panic!("Mount point {} is not a directory", mount_point.display());
        }
        // 绑定挂载
        nix::mount::mount(
            Some(&volume),
            &mount_point,
            None::<&str>,
            nix::mount::MsFlags::MS_BIND,
            None::<&str>,
        ).expect("Bind mount failed");

        info!("Successfully mounted {} to {}", volume.display(), mount_point.display());
    }
}

fn parse_volume(volume: &str) -> (PathBuf, PathBuf) {
    let (volume, mount_point) = volume.split_once(':').expect("Invalid volume format");
    let volume = PathBuf::from(volume);
    let mount_point = PathBuf::from(mount_point);

    if !volume.exists() {
        panic!("Volume {} does not exist", volume.display());
    }

    if !volume.is_dir() {
        panic!("Volume {} is not a directory", volume.display());
    }
    return (volume, mount_point);
}

fn create_lower(container_id: &str, image: &str) {
    let overlayfs = format!("{}{}/{}", ROOTFS_BASE_PATH, container_id, image);
    let image = IMAGE_BASE_PATH.to_string() + image + ".tar";

    if let Err(_) = exists(&image) {
        panic!("cannot find image {}", image);
    }
    
    match exists(&overlayfs) {
        Ok(exists) => {
            if exists {
                info!("Overlayfs {} already exists", overlayfs);
            } else {
                info!("Use image {} to create overlayfs {}", image, overlayfs);
                create_dir_all(&overlayfs).expect("Failed to create directory");
                Command::new("tar")
                    .args(&["-xf", &image, "-C", &overlayfs])
                    .status()
                    .expect("failed to execute tar command");
            }
        }
        Err(_) => {
            panic!("Error checking image existence");
        }
    }
}

// create upper & work
fn create_others(container_id: &str) {
    let others = vec!["upper", "work", "merged"];
    for dir in others {
        let dir_path = format!("{}{}/{}", ROOTFS_BASE_PATH, container_id, dir);
        if !PathBuf::from(&dir_path).exists() { // 检查目录是否存在的方法还挺多
            create_dir_all(&dir_path).expect("Failed to create directory");
        }
    }
}

fn mount_overlayfs(container_id: &str, image: &str) {
    // 完整命令：mount -t overlay overlay -o lowerdir=/root/busybox,upperdir=/root/upper,workdir=/root/work /root/merged
    let root = format!("{}{}", ROOTFS_BASE_PATH, container_id);
    let lower = format!("{}/{}", root, image);
    let upper = format!("{}/upper", root);
    let work = format!("{}/work", root);
    let merged = format!("{}/merged", root);
    let options = format!("lowerdir={},upperdir={},workdir={}", lower, upper, work);
    Command::new("mount")
        .args(&["-t", "overlay", "overlay", "-o", &options, &merged])
        .status()
        .expect("failed to execute mount command");
}

pub fn delete_workspace(container_id: &str, volumn: Option<&str>) {
    if let Some(volumn) = volumn {
        info!("Unmount bind volume {}", volumn);
        let (.., mount_point) = parse_volume(volumn);
        let mount_point = PathBuf::from(format!("{}{}/merged{}", ROOTFS_BASE_PATH, container_id, mount_point.display()));
        nix::mount::umount(&mount_point).expect("Unmount bind volume failed");
    }
    
    let root = format!("{}{}", ROOTFS_BASE_PATH, container_id);
    info!("Deleting overlayfs workspace at {}", root);
    let merged = PathBuf::from(format!("{}/merged", root));
    nix::mount::umount(&merged).expect("Failed to unmount overlayfs");
    let others = vec!["upper", "work", "merged"];
    for dir in others {
        let dir_path = format!("{}/{}", root, dir);
        std::fs::remove_dir_all(dir_path).expect("Failed to remove directory");
    }
}
