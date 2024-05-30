use app::App;
use clap::{Parser, Subcommand};

mod app;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let app = App::new();

    match &cli.command {
        Commands::Ls { name } => {
            app.list(name).await;
        }
        Commands::Up => println!("up"),
        Commands::Down => println!("down"),
        Commands::Delete => println!("delete"),
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Ls { name: Option<String> },
    Up,
    Down,
    Delete,
}
