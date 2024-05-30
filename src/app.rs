use std::collections::HashSet;

use aliyun_oss_client::{types::ObjectQuery, Bucket, Client, EndPoint, Key, Secret};

pub struct App {
    client: Client,
}

impl App {
    pub fn new() -> App {
        App {
            client: init_client(),
        }
    }

    pub async fn list(&self, in_dir: &Option<String>) {
        let mut query = ObjectQuery::new();
        query.insert(ObjectQuery::MAX_KEYS, "20");

        if let Some(in_dir) = in_dir {
            query.insert(ObjectQuery::PREFIX, in_dir);
        }

        let res = Bucket::new("honglei123", EndPoint::CN_SHANGHAI)
            .get_objects(&query, &self.client)
            .await
            .unwrap();

        let list = res.get_vec();

        let mut paths = HashSet::new();
        let mut files = HashSet::new();

        for item in list.iter() {
            let mut file = File::new(item.get_path());
            if let Some(in_dir) = in_dir {
                file = file.sub(in_dir);
            }
            if file.in_dir() {
                paths.insert(file.absolute_dir_nth(1).unwrap());
            } else {
                if file.path.len() > 0 {
                    files.insert(file);
                }
            }
        }
        for item in paths.iter() {
            println!("📂 {}", item);
            println!("");
        }

        for item in files.iter() {
            println!("📄 {}", item.get_path());
            println!("");
        }
        //println!("");
    }
}

pub fn init_client() -> Client {
    use std::env;

    use dotenv::dotenv;

    dotenv().ok();
    let key = env::var("ALIYUN_KEY_ID").unwrap();
    let secret = env::var("ALIYUN_KEY_SECRET").unwrap();

    Client::new(Key::new(key), Secret::new(secret))
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct File {
    path: String,
}

impl File {
    pub fn new<P: Into<String>>(path: P) -> File {
        File { path: path.into() }
    }

    pub fn sub(self, prefix: &str) -> File {
        let prefix = if prefix.chars().last().unwrap() == '/' {
            prefix.to_string()
        } else {
            let mut str = String::from(prefix);
            str.push('/');
            str
        };

        File {
            path: (&self.path[prefix.len()..]).to_string(),
        }
    }

    /// 确认文件是否在目录里面
    pub fn in_dir(&self) -> bool {
        self.path.find('/').is_some()
    }

    /// 获取文件袋各级目录
    pub fn get_dirs(&self) -> Vec<String> {
        let mut dirs: Vec<&str> = self.path.split('/').collect();
        dirs.pop();

        dirs.iter().map(|&d| d.to_owned()).collect()
    }

    /// 根据目录层级，获取绝对路径
    pub fn absolute_dir_nth(&self, num: usize) -> Option<String> {
        let dirs = self.get_dirs();
        if dirs.len() == 0 {
            return None;
        }
        let n = if num > dirs.len() { dirs.len() } else { num };
        let mut dir = String::new();
        for i in 0..n {
            if i == 0 {
                dir.push_str(&dirs[i]);
            } else {
                dir.push('/');
                dir.push_str(&dirs[i]);
            }
        }

        Some(dir)
    }

    pub fn get_path(&self) -> &str {
        &self.path
    }
}
