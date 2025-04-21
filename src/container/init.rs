use libc::{c_void, perror, syscall, SYS_pivot_root};
use log::{info, error};
use nix::mount::{mount, umount2, MntFlags, MsFlags};
use nix::unistd::execvp;
use std::{ffi::CString, path::Path, env::set_current_dir, fs::remove_dir_all};
use std::os::fd::AsRawFd;

use crate::run::{RunArg, ROOTFS_BASE_PATH};
use crate::container::METAINFO_BASE_PATH;

fn setup_mount(container_id: &str) -> Result<(), std::io::Error> {
    mount(None::<&Path>, Path::new("/"), None::<&Path>, MsFlags::MS_PRIVATE | MsFlags::MS_REC, None::<&Path>)?;

    let new_root_str = format!("{}{}/merged", ROOTFS_BASE_PATH, container_id);
    let new_root = Path::new(&new_root_str);
    // pivot_root(new_root, put_old) 的要求之一是：
    // new_root 和 put_old（旧的根目录）必须处于不同的挂载点（mount point）上，也就是：
    // 不能是同一个文件系统（same mount）这是为了避免死循环或“移动自己”这种不可预测的行为。你不能把一个目录挂到它自己内部。
    // 将 new_root 绑定挂载为一个新的 mount point 即可，哪怕它本质上和 old_root 还是在同一个文件系统中
    // 使用 MS_BIND 和 MS_REC 选项来实现这个自己挂载到自己的递归式的绑定挂载
    mount(Some(new_root), new_root, None::<&Path>, MsFlags::MS_BIND | MsFlags::MS_REC, None::<&Path>)?;

    // 新建目录 .old_root
    let old_root_str = format!("{}{}/merged/.old_root", ROOTFS_BASE_PATH, container_id);
    let old_root = Path::new(&old_root_str);
    if !old_root.exists() {
        std::fs::create_dir_all(old_root)?;
    }

    unsafe {
        info!("executing pivot_root, change rootfs");
        let c_new_root = CString::new(new_root_str).unwrap();
        let c_old_root = CString::new(old_root_str).unwrap();
        if syscall(SYS_pivot_root, c_new_root.as_ptr(), c_old_root.as_ptr()) < 0 {
            error!("Error: pivot_root failed");
            perror(std::ptr::null());
            std::process::exit(1);
        }
    }

    // 切换到新的根目录
    set_current_dir("/")?;

    // 卸载旧的根目录
    umount2("/.old_root", MntFlags::MNT_DETACH)?;

    // 删除旧的根目录
    remove_dir_all("/.old_root")?;

    // 挂载 proc 文件系统
    mount(Some("/proc"), "/proc", Some("proc"), MsFlags::empty(), None::<&Path>)?;

    // 挂载 dev 文件系统
    mount(Some("/dev"), "/dev", Some("devtmpfs"), MsFlags::empty(), None::<&Path>)?;
    
    Ok(())
}

fn setup_log(container_id: &str) {
    let log_path = format!("{}{}/container.log", METAINFO_BASE_PATH, container_id);
    info!("Redirecting stdout and stderr to {}", log_path);
    let log_file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&log_path)
        .unwrap_or_else(|e| panic!("Failed to open log file {}: {}", log_path, e));

    let fd = log_file.as_raw_fd();

    unsafe {
        libc::dup2(fd, 1); // stdout
        libc::dup2(fd, 2); // stderr
    }
}

pub extern "C" fn init_process(arg: *mut c_void) -> i32 {
    let run_arg_ref = unsafe { &*(arg as *mut RunArg) };
    info!("Init process started with command {}", run_arg_ref.command);

    if run_arg_ref.detach {
        setup_log(&run_arg_ref.container_id);
    }
    
    match setup_mount(&run_arg_ref.container_id) {
        Ok(_) => {},
        Err(e) => {
            error!("Error: mount setup failed: {}", e);
            return -1;
        }
    }

    let args = std::iter::once(&run_arg_ref.command).chain(run_arg_ref.args.iter())
        .map(|arg| CString::new(arg.clone()).unwrap())
        .collect::<Vec<_>>();
    execvp(&CString::new(run_arg_ref.command.clone()).unwrap(), args.as_slice()).expect("execvp failed");

    return 0;
}