use libc::{mount, umount, mkdir, access};
use log::info;
use crate::utils::*;
use crate::cstr;

// 这里在确定参数类型时从 String、&String 和 &str 中选择了 &str
// String 涉及所有权的转移
// &String 的引用不如 &str 灵活，比如 &str 能接收 "abc" 这样的字符串字面量，而 &String 不能
// &str 还能接收 String 的引用，会自动调用 deref 进行转换
pub fn new_workspace(root: &str) {
    info!("Creating overlayfs workspace at {}", root);
    info!("Create some directories and mount overlayfs to merged.");
    create_lower(root);
    create_others(root);
    mount_overlayfs(root);
}

fn create_lower(root: &str) {
    let busybox_string = root.to_string() + "/busybox\0";
    let busybox = busybox_string.as_str();
    unsafe {
        if access(busybox.as_ptr() as *const i8, libc::F_OK) != 0 {
            check_libc_ret(mkdir(busybox.as_ptr() as *const i8, 0o755), "mkdir failed");
            // 解压 busyboxtar
            check_libc_ret(libc::system(cstr!("tar -xf busybox.tar -C busybox")), "tar failed");
        }
    }
}

// create upper & work
fn create_others(root: &str) {
    let others = vec!["upper\0", "work\0", "merged\0"];
    for dir in others {
        let dir_path = format!("{}/{}", root, dir);
        unsafe {
            if access(dir_path.as_ptr() as *const i8, libc::F_OK) != 0 {
                check_libc_ret(mkdir(dir_path.as_ptr() as *const i8, 0o755), "mkdir failed");
            }
        }
    }
}

fn mount_overlayfs(root: &str) {
    // 完整命令：mount -t overlay overlay -o lowerdir=/root/busybox,upperdir=/root/upper,workdir=/root/work /root/merged
    let lowerdir = format!("{}/busybox", root);
    let upperdir = format!("{}/upper", root);
    let workdir = format!("{}/work", root);
    let overlay = cstr!("overlay");
    let merged = format!("{}/merged\0", root);
    let merged = merged.as_ptr() as *const i8;
    let options = format!("lowerdir={},upperdir={},workdir={}\0", lowerdir, upperdir, workdir);
    let options = options.as_ptr() as *const i8;
    unsafe {
        check_libc_ret(
            mount(overlay, merged, cstr!("overlay"), 0, options as *const libc::c_void),
            "mount overlayfs failed",
        );
    };
}

pub fn delete_workspace(root: &str) {
    info!("Deleting overlayfs workspace at {}", root);
    let merged = format!("{}/merged\0", root);
    let merged = merged.as_ptr() as *const i8;
    unsafe {
        check_libc_ret(umount(merged), "umount overlayfs failed");
    }
    let others = vec!["upper", "work", "merged"];
    for dir in others {
        let dir_path = format!("{}/{}", root, dir);
        std::fs::remove_dir_all(dir_path).expect("Failed to remove directory");
    }
}