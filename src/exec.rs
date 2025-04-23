use libc::{syscall, SYS_pidfd_open};
use nix::sched::{setns, CloneFlags};
use nix::unistd::execvp;
use log::error;
use std::ffi::CString;
use std::os::fd::{FromRawFd, OwnedFd, RawFd};

use crate::ExecCommand;
use crate::container::get_pid;

pub fn exec(command: ExecCommand) {
    let container_id = command.container_id.clone();
    let pid = get_pid(&container_id);
    let pidfd;
    unsafe {
        pidfd = syscall(SYS_pidfd_open, pid, 0);
        if pidfd < 0 {
            error!("Error: pidfd_open failed");
            return;
        }
    }
    let flags = 
        CloneFlags::CLONE_NEWNS | CloneFlags::CLONE_NEWIPC | CloneFlags::CLONE_NEWNET | 
        CloneFlags::CLONE_NEWUTS | CloneFlags::CLONE_NEWPID;
    let pidfd = unsafe {
        OwnedFd::from_raw_fd(pidfd as RawFd)
    };
    setns(pidfd, flags).unwrap_or_else(|e| {
        error!("Error: setns failed: {}", e);
    });
    let args = std::iter::once(&command.command).chain(command.args.iter())
        .map(|arg| CString::new(arg.clone()).unwrap())
        .collect::<Vec<_>>();
    execvp(&CString::new(command.command.clone()).unwrap(), args.as_slice()).expect("execvp failed");
}
