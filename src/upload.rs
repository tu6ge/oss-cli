use std::{path::PathBuf,fs};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
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

    // use std::collections::HashMap;

    // let mut resp_list = Vec::new();

    // for i in 0..4{
    //   let resp = async {
    //     let res = reqwest::get("https://httpbin.org/ip")
    //       .await.unwrap()
    //       .json::<HashMap<String, String>>()
    //       .await.unwrap();
    //     println!("result:{:?}", res);
    //     Ok::<(), String>(())
    //   };
  
    //   resp_list.push(resp);
    // }
    // let res = join_all(resp_list).await;
    
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
    let paths_count = fs::read_dir(self.source).unwrap();

    let total = paths_count.count();

    let base_target = Arc::new(self.target.as_str());

    for path in paths {
      let count = current_count.clone();
      let base_target_copy = base_target.clone();
      
      let task = async move {
        let path = path.unwrap();
        let file = path.metadata().unwrap();
        let file_name = path.file_name();
        let target = String::from(*base_target_copy) + file_name.to_str().unwrap();

        let res = self.client.put_file(path.path(), &target).await;

        let mut count = count.lock().unwrap();
        *count += 1;
        print!("正在上传: {:?}(size:{})   [{}/{}]                         \r", file_name, file.len(), count, total);
        io::stdout().flush().unwrap();
        res
      };
      tasks.push(task);
    }
    
    let _res = join_all(tasks).await;
    println!("上传完成");
  }
}