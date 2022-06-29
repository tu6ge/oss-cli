use std::{path::PathBuf,fs};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use aliyun_oss_client::errors::OssError;
use futures::future::join_all;

use aliyun_oss_client::client::Client;

use crate::errors::{CliResult, CliError};

type Count = Arc<Mutex<u8>>;

pub struct Upload<'a>{
  client: Client<'a>,
  source: &'a PathBuf,
  target: String,
}

impl<'a> Upload<'a> {
  pub fn new(client: Client<'a>, source: &'a PathBuf, target: String) -> Upload<'a> {
    Upload {
      client,
      source,
      target,
    }
  }

  /// 执行上传
  pub async fn upload(&self) -> CliResult<()>{
    if self.source.metadata().unwrap().is_file() {
      self.one_file().await?;
      return Ok(());
    }
    
    self.all_path().await?;
    Ok(())
  }

  /// 单个文件上传
  async fn one_file(&self) -> CliResult<()>{
    print!("正在上传 {:?} (1/1)\r", self.source.file_name().ok_or(CliError::Input("get file name failed"))?);
    let res = self.client.put_file(self.source.to_path_buf(), self.target.as_str()).await;
    match res {
      Ok(_) =>{
        println!("上传成功                                                ");
      }
      _ => {
        println!("上传失败                                                ");
      }
    }

    Ok(())
  }

  /// 整个目录下的文件上传
  async fn all_path(&self) -> CliResult<()>{
    let current_count: Count = Arc::new(Mutex::new(0));
    let mut tasks = Vec::new();

    let paths = fs::read_dir(self.source)?;
    let paths_count = fs::read_dir(self.source)?;

    let total = paths_count.count();

    let base_target = Arc::new(self.target.as_str());

    for path in paths {
      let count = current_count.clone();
      let base_target_copy = base_target.clone();
      
      let task = async move {
        let path = path?;
        let file = path.metadata()?;
        let file_name = path.file_name();
        let target = String::from(*base_target_copy) + file_name.to_str().ok_or(CliError::Input("file_name to_str failed"))?;

        let res = self.client.put_file(path.path(), &target).await?;

        let mut count = count.lock().unwrap();
        *count += 1;
        print!("正在上传: {:?}(size:{})   [{}/{}]                         \r", file_name, file.len(), count, total);
        io::stdout().flush()?;
        Ok::<String, CliError>(res)
      };
      tasks.push(task);
    }
    
    let _res = join_all(tasks).await;
    println!("上传完成");
    Ok(())
  }
}