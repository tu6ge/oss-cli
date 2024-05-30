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

    pub async fn list(&self) {
        let mut query = ObjectQuery::new();
        query.insert(ObjectQuery::MAX_KEYS, "20");
        let res = Bucket::new("honglei123", EndPoint::CN_SHANGHAI)
            .get_objects(&query, &self.client)
            .await
            .unwrap();

        let list = res.get_vec();

        let mut paths = HashSet::new();

        for item in list.iter() {
            if item.in_dir() {
                paths.insert(item.absolute_dir_nth(1).unwrap());
            }
        }
        for item in paths.iter() {
            println!("📂 {}", item);
            println!("");
        }

        for item in list.iter() {
            if !item.in_dir() {
                println!("📄 {}", item.get_path());
                println!("");
            }
        }
        println!("");
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
