use clap::{Args,Parser,Subcommand};
use reqwest::{header::{HeaderMap, HeaderValue},Body};
use std::{path::PathBuf, io::{Read, self, Write}};
use tokio::{fs::File, time::sleep};
use tokio_util::codec::{BytesCodec, FramedRead};
use std::fs;
use std::sync::{Arc, Mutex};
use futures::future::join_all;


use aliyun_oss_client::{client::{Client, ReqeustHandler}, errors::{OssResult, OssError}, auth::VERB};

mod upload;

use upload::Upload;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
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
    }
}

extern crate dotenv;

use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    dotenv().ok();

    match args.command {
        Commands::Up{
            source,
            target,
            key,
            secret,
            endpoint,
            bucket
        } => {
            let key = key.unwrap_or(env::var("ALIYUN_KEY_ID").unwrap());
            let secret = secret.unwrap_or(env::var("ALIYUN_KEY_SECRET").unwrap());
            let endpoint = endpoint.unwrap_or(env::var("ALIYUN_ENDPOINT").unwrap());
            let bucket = bucket.unwrap_or(env::var("ALIYUN_BUCKET").unwrap());
            
            let client = aliyun_oss_client::client(&key,&secret, &endpoint, &bucket);

            let upload = Upload::new(client, &source, target);
            //     let res = client.put_file(source, target.as_str()).await;
            //     println!("res: {res:?}");
            
            upload.action().await;

            

        },
        Commands::Down{ source, target} => {

        }
    }

}

async fn put_content_process(client: &Client<'_>) -> OssResult<String>{
    let file_path = "9AB932LY.jpeg";
    let mut file_size = std::fs::File::open(file_path)?;
    let file = File::open(file_path).await?;
    let mut file_content = Vec::new();
    file_size
    .read_to_end(&mut file_content)?;

    let mut url = client.get_bucket_url()?;
    url.set_path("abc.png");

    let mut headers = HeaderMap::new();
    let content_length = file_content.len().to_string();
    headers.insert(
    "Content-Length", 
    HeaderValue::from_str(&content_length).map_err(|_| OssError::Input("Content-Length parse error".to_string()))?);

    headers.insert(
    "Content-Type", 
    "jpeg".parse().unwrap());

    let response = client.builder(VERB::PUT, &url, Some(headers), None).await?
    .body(file_to_body(file));

    let content = response.send().await?.handle_error()?;

    let result = content.headers().get("ETag")
    .ok_or(OssError::Input("get Etag error".to_string()))?
    .to_str().map_err(|_| OssError::Input("ETag parse error".to_string()))?;

    Ok(result.to_string())
}

fn file_to_body(file: File) -> Body {
    let stream = FramedRead::new(file, BytesCodec::new());
    let body = Body::wrap_stream(stream);
    body
}