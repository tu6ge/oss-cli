
use clap::Parser;
use errors::{CliResult};
use reqwest::{header::{HeaderMap, HeaderValue},Body};
use std::{path::PathBuf, io::{Read}};
use tokio::{fs::File};
use tokio_util::codec::{BytesCodec, FramedRead};

use aliyun_oss_client::{client::{Client, ReqeustHandler}, errors::{OssResult, OssError}, auth::VERB};

mod app;
mod upload;
mod download;
mod errors;

use upload::Upload;
use app::{Cli, Commands};

extern crate dotenv;

use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> CliResult<()>{
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
            let key = key.unwrap_or(env::var("ALIYUN_KEY_ID")?);
            let secret = secret.unwrap_or(env::var("ALIYUN_KEY_SECRET")?);
            let endpoint = endpoint.unwrap_or(env::var("ALIYUN_ENDPOINT")?);
            let bucket = bucket.unwrap_or(env::var("ALIYUN_BUCKET")?);
            
            let client = aliyun_oss_client::client(&key,&secret, &endpoint, &bucket);

            let upload = Upload::new(client, &source, target);
            
            let res = upload.upload().await?;
            println!("upload: {:?}", res);
        },
        Commands::Down{
            source,
            target,
            key,
            secret,
            endpoint,
            bucket
        } => {
            let key = key.unwrap_or(env::var("ALIYUN_KEY_ID")?);
            let secret = secret.unwrap_or(env::var("ALIYUN_KEY_SECRET")?);
            let endpoint = endpoint.unwrap_or(env::var("ALIYUN_ENDPOINT")?);
            let bucket = bucket.unwrap_or(env::var("ALIYUN_BUCKET")?);
            
            let client = aliyun_oss_client::client(&key,&secret, &endpoint, &bucket);
        }
    }

    Ok(())

}

#[allow(unused)]
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