use clap::{Parser, Subcommand};
use chrono::{Datelike, SecondsFormat};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

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

#[derive(Debug, Serialize, Deserialize)]
struct LogEntry {
    timestamp: String,
    message: String,
    tags: Vec<String>,
    decision: bool,
    git: GitContext,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitContext {
    repo: String,
    branch: String,
    commit: String,
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

fn today_log_path() -> PathBuf {
    let now = chrono::Local::now();
    PathBuf::from(".dvlg")
        .join(format!("{:04}", now.year()))
        .join(format!("{:02}", now.month()))
        .join(format!("{:02}.yaml", now.day()))
}

fn run_command(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|err| format!("Failed to run {program}: {err}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let message = if stderr.is_empty() {
            format!("{program} exited with status {}", output.status)
        } else {
            stderr
        };
        return Err(message);
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn collect_git_context() -> GitContext {
    let repo = run_command("git", &["rev-parse", "--show-toplevel"]).unwrap_or_else(|_| "unknown".to_string());
    let branch = run_command("git", &["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_else(|_| "unknown".to_string());
    let commit = run_command("git", &["rev-parse", "HEAD"]).unwrap_or_else(|_| "unknown".to_string());

    GitContext { repo, branch, commit }
}

fn load_entries(path: &PathBuf) -> Result<Vec<LogEntry>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let contents = fs::read_to_string(path)
        .map_err(|err| format!("Failed to read log file {}: {err}", path.display()))?;

    if contents.trim().is_empty() {
        return Ok(Vec::new());
    }

    serde_yaml::from_str(&contents)
        .map_err(|err| format!("Failed to parse log file {}: {err}", path.display()))
}

fn save_entries(path: &PathBuf, entries: &[LogEntry]) -> Result<(), String> {
    let yaml = serde_yaml::to_string(entries)
        .map_err(|err| format!("Failed to serialize log entries: {err}"))?;
    fs::write(path, yaml)
        .map_err(|err| format!("Failed to write log file {}: {err}", path.display()))
}

fn main() {
    let config = load_config();
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
        Commands::Add {
            message,
            tags,
            decision,
        } => {
            let now = chrono::Local::now();
            let path = today_log_path();
            if let Some(parent) = path.parent() {
                if let Err(err) = fs::create_dir_all(parent) {
                    eprintln!("Failed to create log directory: {err}");
                    std::process::exit(1);
                }
            }

            let mut entries = match load_entries(&path) {
                Ok(entries) => entries,
                Err(err) => {
                    eprintln!("{err}");
                    std::process::exit(1);
                }
            };

            let entry = LogEntry {
                timestamp: now.to_rfc3339_opts(SecondsFormat::Secs, true),
                message: message.clone(),
                tags,
                decision,
                git: collect_git_context(),
            };

            entries.push(entry);

            if let Err(err) = save_entries(&path, &entries) {
                eprintln!("{err}");
                std::process::exit(1);
            }

            println!("Added entry to {}", path.display());

            if config.auto_commit.unwrap_or(false) {
                let path_arg = path.to_string_lossy();
                if let Err(err) = run_command("git", &["add", path_arg.as_ref()]) {
                    eprintln!("Failed to stage log file: {err}");
                    std::process::exit(1);
                }

                let commit_message = format!("dvlg add: {message}");
                if let Err(err) = run_command("git", &["commit", "-m", commit_message.as_str()]) {
                    eprintln!("Failed to commit log file: {err}");
                    std::process::exit(1);
                }
            }
        }
        Commands::Today => {
            let path = today_log_path();
            if !path.exists() {
                println!("No entries for today.");
                return;
            }

            let entries = match load_entries(&path) {
                Ok(entries) => entries,
                Err(err) => {
                    eprintln!("{err}");
                    std::process::exit(1);
                }
            };

            let now = chrono::Local::now();
            println!("{:04}-{:02}-{:02}", now.year(), now.month(), now.day());
            for entry in entries {
                if entry.decision {
                    println!("- [decision] {}", entry.message);
                } else {
                    println!("- {}", entry.message);
                }
            }
        }
        Commands::List { .. }
        | Commands::Search { .. }
        | Commands::Export { .. }
        | Commands::Decisions => {
            println!("Not implemented yet.");
        }
    }
}
