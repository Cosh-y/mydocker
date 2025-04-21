use libc::{
    c_void, clone, waitpid, CLONE_NEWIPC, CLONE_NEWNET, CLONE_NEWNS, CLONE_NEWPID, CLONE_NEWUTS, SIGCHLD
};
use log::error;

use crate::container::{delete_workspace, gen_id, init_metainfo, init_process, metainfo_exists, new_workspace, record_exit, record_running};
use crate::RunCommand;
use crate::cgroupsv2::{CGroupManager, ResourceConfig};

pub const IMAGE_BASE_PATH: &str = "/root/.mydocker/image/";         // 镜像存储路径
pub const ROOTFS_BASE_PATH: &str = "/root/.mydocker/overlay2/";     // 镜像以 OverlayFS 的形式 mount 的位置

pub struct RunArg {
    pub container_id: String,
    pub command: String,
    pub args: Vec<String>,
    pub detach: bool,
}

impl RunArg {
    fn new(container_id: &str, command: &str, args: Vec<String>, detach: bool) -> Self {
        RunArg {
            container_id: container_id.to_string(),
            command: command.to_string(),
            args: args,
            detach,
        }
    }
    
}

pub fn run(command: RunCommand) {
    let container_id = gen_id();
    run_container(command, container_id);
}

pub fn run_container(command: RunCommand, container_id: String) {
    let run_arg = Box::new(RunArg::new(&container_id, &command.command, command.args.clone(), command.detach));

    const STACK_SIZE: usize = 1024 * 1024;
    let mut stack = [0; STACK_SIZE];

    let volume: Option<&str> = command.volume.as_deref(); // 获取 volume 的值
    new_workspace(&container_id, &command.image, volume); // 创建 overlayfs 的工作空间，mount volumn 目录

    unsafe {
        let flags = CLONE_NEWPID | CLONE_NEWNS | CLONE_NEWUTS | CLONE_NEWNET | CLONE_NEWIPC | SIGCHLD;
        /*
         * On success, the thread ID of the child process is returned in the
         * caller's thread of execution.  On failure, -1 is returned in the
         * caller's context, no child process is created, and errno is set to
         * indicate the error. [linux man7.org]
         */

        let ret = clone(init_process,   // 使用 libc 中 clone 创建子进程并将子进程放入新的 namespace
            stack.as_mut_ptr().add(STACK_SIZE) as *mut c_void,
            flags,
            Box::into_raw(run_arg) as *mut c_void,
        );
        if ret == -1 {
            error!("Error: clone failed");
        }

        if !metainfo_exists(&container_id) {
            init_metainfo(&container_id, ret as u32, command.clone()); // 初始化容器的元信息
        } else {
            record_running(&container_id, ret as u32); // 记录容器的运行状态
        }

        // let run_arg = RunArg::new(command);
        let cgroupv2_manager = CGroupManager::new(container_id.clone());
        cgroupv2_manager.create_cgroup();
        cgroupv2_manager.set(ResourceConfig {
            cpu: command.cpu,
            memory: command.mem,    // Rust 允许单独移动结构体某个字段的所有权，只要之后不再使用这个字段。
        });
        cgroupv2_manager.add_process(ret as u32); // 将子进程添加到 cgroup 中

        if command.detach {
            return ;
        }
        waitpid(ret, std::ptr::null_mut(), 0); // 等待子进程/容器进程结束
        
        cgroupv2_manager.check_cgroup_memory_events(); // 检查 cgroup 内存事件
        cgroupv2_manager.destroy_cgroup();

        delete_workspace(&container_id, volume); // 删除 overlayfs 的工作空间

        record_exit(&container_id); // 记录容器的退出状态
    }
}
