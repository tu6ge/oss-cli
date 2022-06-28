use std::rc::Rc;
use std::{path::PathBuf,fs};
use core::time;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use futures::executor::block_on;
use futures::future::join_all;

use aliyun_oss_client::client::Client;

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

  pub async fn action(&self){

    if self.source.metadata().unwrap().is_file() {
      self.one_file().await;
      return;
    }

    self.all_path().await;
    
  }

  async fn one_file(&self){
    print!("正在上传 {:?} (1/1)\r", self.source.file_name().unwrap());
    let res = self.client.put_file(self.source.to_path_buf(), self.target.as_str()).await;
    match res {
      Ok(_) =>{
        println!("上传成功                                                ");
      }
      _ => {
        println!("上传失败                                                ");
      }
    }
  }

  async fn all_path(&self){
    let current_count: Count = Arc::new(Mutex::new(0));
    let mut tasks = Vec::new();

    let paths = fs::read_dir(self.source).unwrap();

    let total = 5;

    for path in paths {
      let count = current_count.clone();
      //let f = fs::File::open(path.unwrap().path());
      
      let task = async move{
        let mut count = count.lock().unwrap();
        *count += 1;

        let path = path.unwrap();

        let file = path.metadata().unwrap();
        let file_name = path.file_name();
        print!("正在上传: {:?}({})   [{}/{}]\r", file_name, file.len(), count, total);
        io::stdout().flush().unwrap();

        let target = self.target.clone() + file_name.to_str().unwrap();

        let res = self.client.put_file(path.path(), target.as_str()).await.unwrap();
        res
      };
      tasks.push(task);
    }
    
    futures::executor::block_on(async{
      let res = join_all(tasks).await;
      println!("上传结果:{:?}", res);
    });
    println!("上传完成 [5/5]");
  }
}