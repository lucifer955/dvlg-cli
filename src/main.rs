use clap::{Parser, Subcommand};
use chrono::Datelike;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "dvlg", version, about = "A Git-backed developer diary CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Init,
    Add {
        message: String,
        #[arg(long = "tag")]
        tags: Vec<String>,
        #[arg(long = "decision")]
        decision: bool,
    },
    Today,
    List {
        #[arg(long = "week", conflicts_with = "month")]
        week: bool,
        #[arg(long = "month", conflicts_with = "week")]
        month: bool,
    },
    Search {
        term: String,
    },
    Export {
        #[arg(long = "week", conflicts_with = "month")]
        week: bool,
        #[arg(long = "month", conflicts_with = "week")]
        month: bool,
        #[arg(long = "format", default_value = "markdown")]
        format: String,
    },
    Decisions,
}

#[derive(Debug, Default, Deserialize)]
struct Config {
    auto_commit: Option<bool>,
    default_project: Option<String>,
    editor: Option<String>,
}

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|base| base.join("dvlg").join("config.yaml"))
}

fn load_config() -> Config {
    let path = match config_path() {
        Some(path) => path,
        None => return Config::default(),
    };

    let Ok(contents) = fs::read_to_string(&path) else {
        return Config::default();
    };

    match serde_yaml::from_str(&contents) {
        Ok(config) => config,
        Err(_) => Config::default(),
    }
}

fn main() {
    let _config = load_config();
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            let year = chrono::Local::now().year();
            let path = PathBuf::from(".dvlg").join(year.to_string());
            if path.exists() {
                println!("dvlg already initialized at {}", path.display());
                return;
            }

            if let Err(err) = fs::create_dir_all(&path) {
                eprintln!("Failed to initialize dvlg: {err}");
                std::process::exit(1);
            }

            println!("Initialized dvlg at {}", path.display());
        }
        Commands::Add { .. }
        | Commands::Today
        | Commands::List { .. }
        | Commands::Search { .. }
        | Commands::Export { .. }
        | Commands::Decisions => {
            println!("Not implemented yet.");
        }
    }
}
