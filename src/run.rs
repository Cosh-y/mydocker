use libc::{
    c_void, pipe, clone, write, close, waitpid, CLONE_NEWIPC, CLONE_NEWNET, CLONE_NEWNS, CLONE_NEWPID, CLONE_NEWUTS, SIGCHLD
};
use log::{info, error};

use crate::{container::init_process, DockerSubCmd};
use crate::cgroupsv2::{CGroupManager, ResourceConfig};

pub struct RunArg {
    pub cpu: Option<u32>,
    pub mem: Option<u32>,
    pub image: String,
}

impl RunArg {
    fn new(cmd: DockerSubCmd) -> Self {
        match cmd {
            DockerSubCmd::Run { cpu, mem, image } => RunArg { cpu, mem, image },
        }
    }
    
}

pub fn run(command: DockerSubCmd) {
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
        let ret = clone(init_process, 
            stack.as_mut_ptr().add(STACK_SIZE) as *mut c_void,
            flags,
            Box::into_raw(run_arg) as *mut c_void,
        );
        if ret == -1 {
            error!("Error: clone failed");
        }

        let run_arg = RunArg::new(command);
        let cgroupv2_manager = CGroupManager::new("mydocker".to_string());
        cgroupv2_manager.create_cgroup();
        cgroupv2_manager.set(ResourceConfig {
            cpu: run_arg.cpu,
            memory: run_arg.mem,
        });
        cgroupv2_manager.add_process(ret as u32); // 将子进程添加到 cgroup 中

        waitpid(ret, std::ptr::null_mut(), 0); // 等待子进程/容器进程结束
        
        cgroupv2_manager.destroy_cgroup();
    }
}
