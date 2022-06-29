use std::{path::PathBuf,fs};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use futures::future::join_all;

use aliyun_oss_client::client::Client;

use crate::errors::{CliResult, CliError};

type Count = Arc<Mutex<u8>>;

pub struct Download<'a>{
  client: Client<'a>,
  source: String,
  target: &'a PathBuf,
}