mod run;
mod container;
mod cgroupsv2;

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

        #[arg(long)]        // 对应一个命名选项参数 --mem 20
        mem: Option<u32>,   // Option 表示参数是可选的
        
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
        // DockerSubCmd::Run { image } => run(image),
    }
}
