use crate::StartCommand;
use crate::run::run_container;
use crate::container::{is_running, get_command};

pub fn start(command: StartCommand) {
    let container_id = command.container_id.clone();
    // 检查容器是否已经在运行
    if is_running(&container_id) {
        println!("Container {} is already running", container_id);
        return;
    }

    let run_command = get_command(&container_id);
    run_container(run_command.clone(), container_id.clone());
}