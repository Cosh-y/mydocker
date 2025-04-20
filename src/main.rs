mod run;
mod commit;
mod container;
mod cgroupsv2;
mod utils;

use simple_logger::SimpleLogger;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use run::run;
use container::ps;
use commit::commit_container;

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
}

#[derive(Parser, Clone, Serialize, Deserialize, Debug)]
struct RunCommand {
    #[arg(long)]
    cpu: Option<u32>,
    #[arg(long)]
    mem: Option<String>,
    #[arg(long, short)]
    volume: Option<String>,
    image: String,
    command: String,
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
    }
}
