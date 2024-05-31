use app::App;
use clap::{Parser, Subcommand};

mod app;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let app = App::new();

    match &cli.command {
        Commands::Ls { name } => {
            app.list(name).await;
        }
        Commands::Up { src, dest } => {
            app.upload(src, dest).await;
        }
        Commands::Down { src, dest } => {
            app.download(src, dest).await;
        }
        Commands::Delete { name } => {
            app.delete(name).await;
        }
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 读取文件列表
    Ls {
        /// 要读取的目录
        name: Option<String>,
    },

    /// 上传文件
    Up {
        /// 原文件路径
        src: String,
        /// OSS 路径
        dest: String,
    },

    /// 下载文件
    Down {
        /// OSS 路径
        src: String,
        /// 本地的相对路径
        dest: String,
    },

    /// 删除文件
    Delete {
        /// 文件的完整路径
        name: String,
    },
}
