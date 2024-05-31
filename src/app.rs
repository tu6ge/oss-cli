use std::collections::HashSet;

use aliyun_oss_client::{types::ObjectQuery, Bucket, Client, EndPoint, Key, Object, Secret};
use chrono::{DateTime, Utc};
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::{CrosstermBackend, Stylize, Terminal},
    widgets::Paragraph,
};
use serde::Deserialize;
use std::io::{stdout, Result, Write};

#[derive(Clone)]
pub struct App {
    client: Client,
    pub next_token: Option<String>,
    pub list_content: Option<String>,
    pub is_last: bool,
}

impl App {
    pub fn new() -> App {
        let client = init_client();
        App {
            client,
            next_token: None,
            list_content: None,
            is_last: false,
        }
    }

    pub fn get_list_content(&self) -> Option<String> {
        self.list_content.clone()
    }

    pub async fn list_page(&mut self, in_dir: &Option<String>) -> Result<()> {
        stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        terminal.clear()?;

        self.list(in_dir, None).await;

        loop {
            let content = self.get_list_content().unwrap();
            terminal.draw(|frame| {
                let layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(vec![Constraint::Percentage(90), Constraint::Percentage(10)])
                    .split(frame.size());
                frame.render_widget(
                    Paragraph::new(content.clone()).white().on_black(),
                    layout[0],
                );

                if self.is_last {
                    frame.render_widget(
                        Paragraph::new("已经是最后一页了，按 q 退出")
                            .white()
                            .on_black()
                            .centered(),
                        layout[1],
                    );
                } else {
                    frame.render_widget(
                        Paragraph::new("按 s 查询下一页，按 q 退出")
                            .white()
                            .on_black()
                            .centered(),
                        layout[1],
                    );
                }
            })?;
            if event::poll(std::time::Duration::from_millis(16))? {
                if let event::Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                        break;
                    }
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('s') {
                        if !self.is_last {
                            self.list(in_dir, self.next_token.clone()).await;
                        }
                    }
                }
            }
        }

        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;

        Ok(())
    }

    pub async fn list(&mut self, in_dir: &Option<String>, next_token: Option<String>) {
        use term_grid::{Cell, Direction, Filling, Grid, GridOptions};

        let mut query = ObjectQuery::new();
        query.insert(ObjectQuery::MAX_KEYS, "30");

        if let Some(in_dir) = in_dir {
            query.insert(ObjectQuery::PREFIX, in_dir);
        }
        if let Some(token) = next_token {
            query.insert(ObjectQuery::CONTINUATION_TOKEN, token);
        }

        let (res, token): (Vec<ListObject>, _) = Bucket::new("honglei123", EndPoint::CN_SHANGHAI)
            .export_objects(&query, &self.client)
            .await
            .unwrap();
        if let None = token {
            self.is_last = true;
        } else {
            self.next_token = token;
        }

        let list = res;

        let mut paths = HashSet::new();
        let mut files = HashSet::new();

        for item in list.iter() {
            let mut file = File::new(&item.path, &item.date);
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
        let mut grid = Grid::new(GridOptions {
            filling: Filling::Spaces(3),
            direction: Direction::LeftToRight,
        });
        grid.add(Cell::from(format!("文件名")));
        grid.add(Cell::from(format!("修改时间")));

        for item in paths.iter() {
            grid.add(Cell::from(format!("📂 {}", item)));
            grid.add(Cell::from(""));
        }

        for item in files.iter() {
            grid.add(Cell::from(format!("📄 {}", item.get_path())));
            grid.add(Cell::from(item.date()));
        }
        self.list_content = Some(format!("{}", grid.fit_into_columns(2)));
    }

    pub async fn upload(&self, src: &str, dest: &str) {
        let current_dir = std::env::current_dir().expect("获取当前目录失败");
        let file_path = current_dir.join(src);

        let content = std::fs::read_to_string(file_path).expect("读取文件失败");

        let content_vec = content.into_bytes();

        let obj = Object::new(dest);

        obj.upload(content_vec, &self.client)
            .await
            .expect("上传失败");

        println!("上传成功");
    }

    pub async fn download(&self, src: &str, dest: &str) {
        let obj = Object::new(src);
        let vec = obj.download(&self.client).await.expect("下载文件失败");

        let current_dir = std::env::current_dir().expect("获取当前目录失败");
        let file_path = current_dir.join(dest);
        let mut file = std::fs::File::create(file_path).expect("文件创建失败");
        file.write_all(&vec).expect("文件写入内容失败");

        println!("下载成功");
    }

    pub async fn delete(&self, name: &str) {
        let obj = Object::new(name);
        obj.delete(&self.client).await.expect("删除失败");

        println!("删除成功");
    }
}

pub fn init_client() -> Client {
    use std::env;

    use dotenv::dotenv;

    dotenv().ok();
    let key = env::var("ALIYUN_KEY_ID").expect("未设置 ALIYUN_KEY_ID 环境变量");
    let secret = env::var("ALIYUN_KEY_SECRET").expect("未设置 ALIYUN_KEY_SECRET 环境变量");
    let endpoint = env::var("ALIYUN_ENDPOINT").expect("未设置 ALIYUN_ENDPOINT 环境变量");
    let bucket = env::var("ALIYUN_BUCKET").expect("未设置 ALIYUN_BUCKET 环境变量");

    let mut client = Client::new(Key::new(key), Secret::new(secret));
    client.set_bucket(Bucket::new(
        bucket,
        EndPoint::new(&endpoint).expect("找不到匹配的 endpoint"),
    ));
    client
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct File {
    path: String,
    date: DateTime<Utc>,
}

impl File {
    pub fn new<P: Into<String>>(path: P, date: &str) -> File {
        let date: DateTime<Utc> = date.parse().unwrap();
        File {
            path: path.into(),
            date,
        }
    }

    fn date(&self) -> String {
        format!("{}", self.date.format("%Y-%m-%d %H:%M"))
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
            date: self.date,
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

#[derive(Debug, Deserialize)]
struct ListObject {
    #[serde(rename = "Key")]
    path: String,

    #[serde(rename = "LastModified")]
    date: String,
}
