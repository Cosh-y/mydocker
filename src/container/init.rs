use libc::{access, c_void, chdir, execvp, mkdir, mount, perror, rmdir, syscall, umount2, SYS_pivot_root, MNT_DETACH, MS_BIND, MS_PRIVATE, MS_REC};
use log::{info, error};

use crate::run::RunArg;

fn setup_mount() {
    unsafe {
        if mount("\0".as_ptr() as *const i8, 
                "/\0".as_ptr() as *const i8,
                "\0".as_ptr() as *const i8,
                MS_PRIVATE | MS_REC,
                std::ptr::null()
            ) < 0 {
            error!("Error: mount failed");
            perror(std::ptr::null());
            return ;
        }

        let new_root = "./busybox\0".as_ptr() as *const i8;
        // pivot_root(new_root, put_old) 的要求之一是：
        // new_root 和 put_old（旧的根目录）必须处于不同的挂载点（mount point）上，也就是：
        // 不能是同一个文件系统（same mount）这是为了避免死循环或“移动自己”这种不可预测的行为。你不能把一个目录挂到它自己内部。
        // 将 new_root 绑定挂载为一个新的 mount point 即可，哪怕它本质上和 old_root 还是在同一个文件系统中
        // 使用 MS_BIND 和 MS_REC 选项来实现这个自己挂载到自己的递归式的绑定挂载
        if mount(new_root, new_root, "\0".as_ptr() as *const i8, MS_BIND | MS_REC, std::ptr::null()) < 0 {
            error!("Error: mount failed");
            perror(std::ptr::null());
            return ;
        }

        // 新建目录 .old_root
        let old_root = "./busybox/.old_root\0".as_ptr() as *const i8;
        if access(old_root, libc::F_OK) != 0 {
            if mkdir(old_root, 0o755) < 0 {
                error!("Error: mkdir failed");
                perror(std::ptr::null());
                return ;
            }
        }

        if syscall(SYS_pivot_root, new_root, old_root) < 0 {
            error!("Error: pivot_root failed");
            perror(std::ptr::null());
            return ;
        }

        // 切换到新的根目录
        if chdir("/\0".as_ptr() as *const i8) < 0 {
            error!("Error: chdir failed");
            perror(std::ptr::null());
            return ;
        }

        // 查看当前的工作目录
        let current_working_dir = std::env::current_dir().unwrap();
        info!("Current working directory: {:?}", current_working_dir);

        // 卸载旧的根目录
        let old_root = "/.old_root\0".as_ptr() as *const i8; // 这里的 old_root 是新的根目录下的 .old_root 目录
        if umount2(old_root, MNT_DETACH) < 0 {
            error!("Error: umount failed");
            perror(std::ptr::null());
            return ;
        }

        // 删除旧的根目录
        if rmdir(old_root) < 0 {
            error!("Error: rmdir failed");
            perror(std::ptr::null());
            return ;
        }

        // 挂载 proc 文件系统
        if mount("proc\0".as_ptr() as *const i8, 
                        "/proc\0".as_ptr() as *const i8, 
                        "proc\0".as_ptr() as *const i8, 
                        0, 
                        std::ptr::null()
                    ) < 0 {
            error!("Error: mount failed");
            perror(std::ptr::null());
            return ;
        }

        // 挂载 dev 文件系统
        if mount("devtmpfs\0".as_ptr() as *const i8, 
                        "/dev\0".as_ptr() as *const i8, 
                        "devtmpfs\0".as_ptr() as *const i8, 
                        0, 
                        std::ptr::null()
                    ) < 0 {
            error!("Error: mount failed");
            perror(std::ptr::null());
            return ;
        }
    }
}

pub extern "C" fn init_process(arg: *mut c_void) -> i32 {
    let run_arg_ref = unsafe { &*(arg as *mut RunArg) };
    info!("Init process started with args: image {} cpu {}", run_arg_ref.image, run_arg_ref.cpu.unwrap_or(0));
    
    setup_mount();

    unsafe {
        let argv = [run_arg_ref.image.as_ptr() as *const i8, std::ptr::null()];
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