mod run;
mod container;
mod cgroupsv2;
mod utils;

use simple_logger::SimpleLogger;
use clap::{Parser, Subcommand};
use run::run;

#[derive(Parser)]
#[command(author)]
struct Cli {
    #[command(subcommand)]
    subcommand: DockerSubCmd,
}

#[derive(Subcommand, Clone)]
enum DockerSubCmd {
    Run(RunCommand),
}

#[derive(Parser, Clone, Debug)]
struct RunCommand {
    #[arg(long)]
    cpu: Option<u32>,
    #[arg(long)]
    mem: Option<String>,
    #[arg(long, short)]
    volume: Option<String>,
    image: String,
}

fn main() {
    SimpleLogger::new().init().unwrap();
    let cli = Cli::parse();
    match cli.subcommand {
        DockerSubCmd::Run(run_command) => {
            run(run_command);
        }
    }
}
