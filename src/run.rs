use libc::{
    c_void, clone, waitpid, CLONE_NEWIPC, CLONE_NEWNET, CLONE_NEWNS, CLONE_NEWPID, CLONE_NEWUTS, SIGCHLD
};
use log::error;

use crate::container::{init_process, new_workspace, delete_workspace, init_metainfo, record_exit};
use crate::RunCommand;
use crate::cgroupsv2::{CGroupManager, ResourceConfig};

pub static ROOTFS: &str = ".";

pub struct RunArg {
    pub image: String,
    pub rootfs: String,
}

impl RunArg {
    fn new(cmd: RunCommand) -> Self {
        RunArg {
            image: cmd.image,
            rootfs: String::from(ROOTFS),
        }
    }
    
}

pub fn run(command: RunCommand) {
    
    let run_arg = Box::new(RunArg::new(command.clone()));

    const STACK_SIZE: usize = 1024 * 1024;
    let mut stack = [0; STACK_SIZE];

    unsafe {
        let flags = CLONE_NEWPID | CLONE_NEWNS | CLONE_NEWUTS | CLONE_NEWNET | CLONE_NEWIPC | SIGCHLD;
        /*
         * On success, the thread ID of the child process is returned in the
         * caller's thread of execution.  On failure, -1 is returned in the
         * caller's context, no child process is created, and errno is set to
         * indicate the error. [linux man7.org]
         */

        let volume: Option<&str> = command.volume.as_deref(); // 获取 volume 的值
        new_workspace(ROOTFS, volume); // 创建 overlayfs 的工作空间

        let ret = clone(init_process,   // 使用 libc 中 clone 创建子进程并将子进程放入新的 namespace
            stack.as_mut_ptr().add(STACK_SIZE) as *mut c_void,
            flags,
            Box::into_raw(run_arg) as *mut c_void,
        );
        if ret == -1 {
            error!("Error: clone failed");
        }

        let container_id = init_metainfo(ret as u32, command.clone()); // 初始化容器的元信息

        // let run_arg = RunArg::new(command);
        let cgroupv2_manager = CGroupManager::new("mydocker".to_string());
        cgroupv2_manager.create_cgroup();
        cgroupv2_manager.set(ResourceConfig {
            cpu: command.cpu,
            memory: command.mem,    // Rust 允许单独移动结构体某个字段的所有权，只要之后不再使用这个字段。
        });
        cgroupv2_manager.add_process(ret as u32); // 将子进程添加到 cgroup 中

        waitpid(ret, std::ptr::null_mut(), 0); // 等待子进程/容器进程结束
        
        cgroupv2_manager.check_cgroup_memory_events(); // 检查 cgroup 内存事件
        cgroupv2_manager.destroy_cgroup();

        delete_workspace(ROOTFS, volume); // 删除 overlayfs 的工作空间

        record_exit(container_id); // 记录容器的退出状态
    }
}
