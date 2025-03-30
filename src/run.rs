use libc::{
    c_void, clone, waitpid, CLONE_NEWIPC, CLONE_NEWNET, CLONE_NEWNS, CLONE_NEWPID, CLONE_NEWUTS, SIGCHLD
};

use crate::container::init_process; // what?? what is the rule of mod and use exactly!?

pub fn run(command: String) {
    println!("Run image {}", command);
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
            command.as_ptr() as *mut c_void
        );
        if ret == -1 {
            println!("Error: clone failed");
        }
        waitpid(ret, std::ptr::null_mut(), 0);
    }
}
