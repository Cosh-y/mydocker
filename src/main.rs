mod run;

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
    Init,
    Run {
        image: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.subcommand {
        DockerSubCmd::Init => println!("Init command not yet implemented"),
        DockerSubCmd::Run { image } => run(image),
    }
}
