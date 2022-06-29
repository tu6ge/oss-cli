use std::path::PathBuf;

use clap::{Args,Parser,Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[clap(arg_required_else_help = true)]
    Up{
        /// 本地文件路径
        #[clap(value_parser)]
        source: PathBuf,

        /// OSS 文件 key
        #[clap(value_parser)]
        target: String,

        #[clap(short, long, value_parser)]
        key: Option<String>,
        #[clap(short, long, value_parser)]
        secret: Option<String>,
        #[clap(short, long, value_parser)]
        endpoint: Option<String>,
        #[clap(short, long, value_parser)]
        bucket: Option<String>,
    },
    #[clap(arg_required_else_help = true)]
    Down{
        /// OSS 文件 key
        #[clap(value_parser)]
        source: String,

        /// 本地文件路径
        #[clap(value_parser)]
        target: PathBuf,

        #[clap(short, long, value_parser)]
        key: Option<String>,
        #[clap(short, long, value_parser)]
        secret: Option<String>,
        #[clap(short, long, value_parser)]
        endpoint: Option<String>,
        #[clap(short, long, value_parser)]
        bucket: Option<String>,
    }
}