mod run;
mod container;

use simple_logger::SimpleLogger;
use clap::{Parser, Subcommand};
use run::run;

#[derive(Parser)]
#[command(author)]
struct Cli {
    #[command(subcommand)]
    subcommand: DockerSubCmd,
}

#[derive(Subcommand)]
enum DockerSubCmd {
    Run {
        image: String,
    },
}

fn main() {
    SimpleLogger::new().init().unwrap();
    let cli = Cli::parse();
    match cli.subcommand {
        DockerSubCmd::Run { image } => run(image),
    }
}
