use libc::{c_void, execvp, perror, syscall, SYS_pivot_root};
use log::{info, error};
use nix::mount::{mount, umount2, MntFlags, MsFlags};
use std::{ffi::CString, path::Path, env::set_current_dir, fs::remove_dir_all};

use crate::run::{RunArg, ROOTFS_BASE_PATH};

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

pub extern "C" fn init_process(arg: *mut c_void) -> i32 {
    let run_arg_ref = unsafe { &*(arg as *mut RunArg) };
    info!("Init process started with command {}", run_arg_ref.command);
    
    match setup_mount(run_arg_ref.container_id.as_str()) {
        Ok(_) => {},
        Err(e) => {
            error!("Error: mount setup failed: {}", e);
            return -1;
        }
    }

    unsafe {
        let c_command = CString::new(run_arg_ref.command.clone()).unwrap();
        let argv = [c_command.as_ptr(), std::ptr::null()];
        // execvp 的第一个参数是要加载执行的可执行文件的路径，第二个参数是一个字符串数组，表示要传递给可执行文件的参数列表
        // 当使用用 busybox 构成的 rootfs 时，无论第一个参数是 /bin/ls 还是 /bin/pwd 执行的都是 busybox
        // 他们实际上是 /bin/busybox 的硬链接
        // 而 /bin/busybox 会读取 argv[0] 的值来决定执行哪个命令，比如 argv[0] 是 /bin/ls 时，busybox 就会执行 ls 命令
        let ret = execvp("/bin/busybox\0".as_ptr() as *const i8, argv.as_ptr() as *const *const i8);
        if ret < 0 {
            error!("Error: exec failed");
            perror(std::ptr::null());
            return -1;
        }
    }
    return 0;
}