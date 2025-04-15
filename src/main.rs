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
    Run {
        #[arg(long)]        // 对应一个命名选项参数 --cpu 20
        cpu: Option<u32>,   // Option 表示参数是可选的

        #[arg(long)]        // 对应一个命名选项参数 --mem 10m
        mem: Option<String>,   // Option 表示参数是可选的
        
        image: String,      // 对应一个固定位置参数
    },
}

fn main() {
    SimpleLogger::new().init().unwrap();
    let cli = Cli::parse();
    match cli.subcommand {
        DockerSubCmd::Run { .. } => {
            run(cli.subcommand);
        }
    }
}
