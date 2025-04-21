mod run;
mod commit;
mod container;
mod cgroupsv2;
mod rm;
mod start;
mod stop;
mod mydocker_log;

use simple_logger::SimpleLogger;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use run::run;
use container::ps;
use commit::commit_container;
use start::start;
use stop::stop;
use rm::rm;
use mydocker_log::log;
#[derive(Parser)]
#[command(author)]
struct Cli {
    #[command(subcommand)]
    subcommand: DockerSubCmd,
}

#[derive(Subcommand)]
enum DockerSubCmd {
    Run(RunCommand),
    Commit(CommitCommand),
    Ps(PsCommand),
    Stop(StopCommand),
    Start(StartCommand),
    Rm(RmCommand),
    Log(LogCommand),
}

#[derive(Parser, Clone, Serialize, Deserialize, Debug)]
struct RunCommand {
    #[arg(long)]
    cpu: Option<u32>,
    #[arg(long)]
    mem: Option<String>,
    #[arg(long, short)]
    volume: Option<String>,
    #[arg(long, short)]
    detach: bool,
    image: String,
    command: String,
    args: Vec<String>,
}

#[derive(Parser)]
struct CommitCommand {
    container_id: String,
    image: String,
}

#[derive(Parser)]
struct PsCommand {
    #[arg(long, short)]
    all: bool,
}

#[derive(Parser)]
struct StopCommand {
    container_id: String,
}

#[derive(Parser)]
struct StartCommand {
    container_id: String,
}

#[derive(Parser)]
struct RmCommand {
    container_id: String,
}

#[derive(Parser)]
struct LogCommand {
    container_id: String,
}

fn main() {
    SimpleLogger::new().init().unwrap();
    let cli = Cli::parse();
    match cli.subcommand {
        DockerSubCmd::Run(run_command) => {
            run(run_command);
        },
        DockerSubCmd::Commit(commit_command) => {
            commit_container(&commit_command.container_id, &commit_command.image);
        },
        DockerSubCmd::Ps(ps_command) => {
            ps(ps_command);
        },
        DockerSubCmd::Stop(stop_command) => {
            stop(stop_command);
        },
        DockerSubCmd::Start(start_command) => {
            start(start_command);
        },
        DockerSubCmd::Rm(rm_command) => {
            rm(rm_command);
        },
        DockerSubCmd::Log(log_command) => {
            log(&log_command.container_id);
        },
    }
}
