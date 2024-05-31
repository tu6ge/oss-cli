use app::App;
use clap::{Parser, Subcommand};
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
use std::io::{stdout, Result};

mod app;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut app = App::new();

    match &cli.command {
        Commands::Ls { name } => {
            stdout().execute(EnterAlternateScreen)?;
            enable_raw_mode()?;
            let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
            terminal.clear()?;

            // TODO main loop
            app.list(name, None).await;

            loop {
                let content = app.get_list_content().unwrap();
                terminal.draw(|frame| {
                    let layout = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints(vec![Constraint::Percentage(90), Constraint::Percentage(10)])
                        .split(frame.size());
                    frame.render_widget(
                        Paragraph::new(content.clone()).white().on_black(),
                        layout[0],
                    );

                    if app.is_last {
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
                            if !app.is_last {
                                app.list(name, app.next_token.clone()).await;
                            }
                        }
                    }
                }
            }

            stdout().execute(LeaveAlternateScreen)?;
            disable_raw_mode()?;
        }
        Commands::Up { src, dest } => {
            app.upload(src, dest).await;
        }
        Commands::Down { src, dest } => {
            app.download(src, dest).await;
        }
        Commands::Delete { name } => {
            app.delete(name).await;
        }
    }

    Ok(())
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 读取文件列表
    Ls {
        /// 要读取的目录
        name: Option<String>,
    },

    /// 上传文件
    Up {
        /// 原文件路径
        src: String,
        /// OSS 路径
        dest: String,
    },

    /// 下载文件
    Down {
        /// OSS 路径
        src: String,
        /// 本地的相对路径
        dest: String,
    },

    /// 删除文件
    Delete {
        /// 文件的完整路径
        name: String,
    },
}
