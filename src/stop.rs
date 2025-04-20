use log::{error, info};
use nix::sys::signal::{kill, Signal};
use nix::unistd::{sleep, Pid};

use crate::StopCommand;
use crate::container::{delete_workspace, get_pid, get_volume, is_running, record_exit};
use crate::cgroupsv2::CGroupManager;

pub fn stop(command: StopCommand) {
    let container_id = command.container_id.clone();
    if !is_running(&container_id) {
        error!("Container {} is not running", container_id);
        return;
    }
    let pid = get_pid(&container_id);
    info!("Stopping container {} with PID {}", container_id, pid);
    // 使用 SIGTERM 信号停止容器
    if let Err(e) = kill(Pid::from_raw(pid as i32), Signal::SIGTERM) {
        error!("Failed to stop container {}: {}", container_id, e);
        return;
    }

    sleep(5);
    // 检查容器是否已经停止
    if let Ok(_) = kill(Pid::from_raw(pid as i32), None) {
        kill(Pid::from_raw(pid as i32), Signal::SIGKILL).unwrap();
        info!("Container process {} is still running, forcefully killed", pid);
    }

    // 删除容器 overlayfs 目录
    let volume = get_volume(&container_id);
    delete_workspace(&container_id, volume.as_deref());

    // 删除容器 cgroup 目录
    let cgroupv2_manager = CGroupManager::new(container_id.clone());
    cgroupv2_manager.destroy_cgroup();

    // 记录容器 exited 状态
    record_exit(&container_id);
}