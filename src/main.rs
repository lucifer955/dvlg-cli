#![allow(dead_code)]

use clap::{Parser, Subcommand};
use chrono::{Datelike, SecondsFormat};
use chrono::Duration;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::env;
use walkdir::WalkDir;

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
    Config {
        #[command(subcommand)]
        action: ConfigCommand,
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

#[derive(Subcommand, Debug)]
enum ConfigCommand {
    Show,
    Set {
        #[arg(long = "log-repo-path")]
        log_repo_path: Option<String>,
        #[arg(long = "auto-commit")]
        auto_commit: Option<bool>,
    },
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[allow(dead_code)]
struct Config {
    auto_commit: Option<bool>,
    #[allow(dead_code)]
    default_project: Option<String>,
    #[allow(dead_code)]
    editor: Option<String>,
    log_repo_path: Option<String>,
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

fn save_config(config: &Config) -> Result<(), String> {
    let path = config_path().ok_or_else(|| "Failed to resolve config path".to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("Failed to create config directory: {err}"))?;
    }

    let yaml = serde_yaml::to_string(config)
        .map_err(|err| format!("Failed to serialize config: {err}"))?;
    fs::write(&path, yaml)
        .map_err(|err| format!("Failed to write config file {}: {err}", path.display()))
}

fn log_root(config: &Config) -> PathBuf {
    if let Some(path) = &config.log_repo_path {
        PathBuf::from(path)
    } else {
        env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    }
}

fn log_store_root(config: &Config) -> PathBuf {
    log_root(config).join(".dvlg")
}

fn ensure_initialized(config: &Config) {
    if !log_store_root(config).exists() {
        eprintln!("dvlg is not initialized. Run 'dvlg init'.");
        std::process::exit(1);
    }
}

fn today_log_path(config: &Config) -> PathBuf {
    let now = chrono::Local::now();
    log_store_root(config)
        .join(format!("{:04}", now.year()))
        .join(format!("{:02}", now.month()))
        .join(format!("{:02}.yaml", now.day()))
}

fn log_path_for_date(config: &Config, date: chrono::NaiveDate) -> PathBuf {
    log_store_root(config)
        .join(format!("{:04}", date.year()))
        .join(format!("{:02}", date.month()))
        .join(format!("{:02}.yaml", date.day()))
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

fn format_date_from_path(path: &std::path::Path) -> Option<String> {
    let file_stem = path.file_stem()?.to_string_lossy();
    let day = file_stem.parse::<u32>().ok()?;
    let month = path.parent()?.file_name()?.to_string_lossy().parse::<u32>().ok()?;
    let year = path
        .parent()?
        .parent()?
        .file_name()?
        .to_string_lossy()
        .parse::<i32>()
        .ok()?;

    Some(format!("{:04}-{:02}-{:02}", year, month, day))
}

fn collect_entries_for_range(
    config: &Config,
    start: chrono::NaiveDate,
    end: chrono::NaiveDate,
) -> Vec<(chrono::NaiveDate, Vec<LogEntry>)> {
    let mut result = Vec::new();
    let mut current = start;
    while current <= end {
        let path = log_path_for_date(config, current);
        if path.exists() {
            if let Ok(entries) = load_entries(&path) {
                if !entries.is_empty() {
                    result.push((current, entries));
                }
            }
        }
        current = current + Duration::days(1);
    }
    result
}

fn render_export_markdown(entries: &[(chrono::NaiveDate, Vec<LogEntry>)]) {
    for (date, items) in entries {
        println!("## {:04}-{:02}-{:02}", date.year(), date.month(), date.day());
        for item in items {
            if item.decision {
                println!("- **[decision]** {}", item.message);
            } else {
                println!("- {}", item.message);
            }
        }
        println!();
    }
}

fn render_export_text(entries: &[(chrono::NaiveDate, Vec<LogEntry>)]) {
    for (date, items) in entries {
        println!("{:04}-{:02}-{:02}", date.year(), date.month(), date.day());
        for item in items {
            if item.decision {
                println!("* [decision] {}", item.message);
            } else {
                println!("* {}", item.message);
            }
        }
        println!();
    }
}

fn main() {
    let config = load_config();
    let _ = &config.default_project;
    let _ = &config.editor;
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            let root = log_root(&config);
            if !root.exists() {
                eprintln!("Log repo path does not exist: {}", root.display());
                std::process::exit(1);
            }

            let year = chrono::Local::now().year();
            let path = log_store_root(&config).join(year.to_string());
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
            let path = today_log_path(&config);
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
                let root = log_root(&config);
                let root_arg = root.to_string_lossy();
                let add_path = path.strip_prefix(&root).unwrap_or(&path);
                let add_arg = add_path.to_string_lossy();
                if let Err(err) = run_command("git", &["-C", root_arg.as_ref(), "add", add_arg.as_ref()]) {
                    eprintln!("Failed to stage log file: {err}");
                    std::process::exit(1);
                }

                let commit_message = format!("dvlg add: {message}");
                if let Err(err) = run_command("git", &["-C", root_arg.as_ref(), "commit", "-m", commit_message.as_str()]) {
                    eprintln!("Failed to commit log file: {err}");
                    std::process::exit(1);
                }
            }
        }
        Commands::Config { action } => {
            match action {
                ConfigCommand::Show => {
                    let path = config_path().unwrap_or_else(|| PathBuf::from("<unknown>"));
                    println!("Config file: {}", path.display());
                    let yaml = serde_yaml::to_string(&config)
                        .unwrap_or_else(|_| "<invalid config>".to_string());
                    print!("{yaml}");
                }
                ConfigCommand::Set {
                    log_repo_path,
                    auto_commit,
                } => {
                    let mut updated = config;
                    if let Some(path) = log_repo_path {
                        updated.log_repo_path = Some(path);
                    }
                    if let Some(value) = auto_commit {
                        updated.auto_commit = Some(value);
                    }

                    if let Err(err) = save_config(&updated) {
                        eprintln!("{err}");
                        std::process::exit(1);
                    }

                    println!("Config updated.");
                }
            }
        }
        Commands::Today => {
            ensure_initialized(&config);
            let path = today_log_path(&config);
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
        Commands::List { week, month } => {
            ensure_initialized(&config);
            let today = chrono::Local::now().date_naive();
            let start = if week {
                today - Duration::days(6)
            } else if month {
                chrono::NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
                    .unwrap_or(today)
            } else {
                today
            };

            let mut current = start;
            while current <= today {
                let path = log_path_for_date(&config, current);
                if path.exists() {
                    let entries = match load_entries(&path) {
                        Ok(entries) => entries,
                        Err(err) => {
                            eprintln!("{err}");
                            std::process::exit(1);
                        }
                    };

                    for entry in entries {
                        println!("{:04}-{:02}-{:02} - {}", current.year(), current.month(), current.day(), entry.message);
                    }
                }

                current = current + Duration::days(1);
            }
        }
        Commands::Search { term } => {
            ensure_initialized(&config);
            let needle = term.to_lowercase();
            for entry in WalkDir::new(log_store_root(&config))
                .into_iter()
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.file_type().is_file())
                .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("yaml"))
            {
                let path = entry.path().to_path_buf();
                let entries = match load_entries(&path) {
                    Ok(entries) => entries,
                    Err(err) => {
                        eprintln!("{err}");
                        std::process::exit(1);
                    }
                };

                let date = format_date_from_path(&path).unwrap_or_else(|| path.display().to_string());
                for item in entries {
                    let message = item.message.to_lowercase();
                    let tags = item.tags.join(" ").to_lowercase();
                    if message.contains(&needle) || tags.contains(&needle) {
                        println!("{} - {}", date, item.message);
                    }
                }
            }
        }
        Commands::Export { week, month, format } => {
            ensure_initialized(&config);
            let today = chrono::Local::now().date_naive();
            let start = if week {
                today - Duration::days(6)
            } else if month {
                chrono::NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
                    .unwrap_or(today)
            } else {
                today
            };

            let entries = collect_entries_for_range(&config, start, today);
            if entries.is_empty() {
                println!("No entries found.");
                return;
            }

            match format.as_str() {
                "markdown" => render_export_markdown(&entries),
                "text" => render_export_text(&entries),
                _ => {
                    eprintln!("Unsupported format: {format}");
                    std::process::exit(1);
                }
            }
        }
        | Commands::Decisions => {
            ensure_initialized(&config);
            let mut results: Vec<(String, String)> = Vec::new();

            for entry in WalkDir::new(log_store_root(&config))
                .into_iter()
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.file_type().is_file())
                .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("yaml"))
            {
                let path = entry.path().to_path_buf();
                let entries = match load_entries(&path) {
                    Ok(entries) => entries,
                    Err(err) => {
                        eprintln!("{err}");
                        std::process::exit(1);
                    }
                };

                let date = format_date_from_path(&path).unwrap_or_else(|| path.display().to_string());
                for item in entries {
                    if item.decision {
                        results.push((date.clone(), item.message));
                    }
                }
            }

            results.sort_by(|a, b| a.0.cmp(&b.0));
            if results.is_empty() {
                println!("No decisions found.");
                return;
            }

            for (date, message) in results {
                println!("{} - {}", date, message);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn fixture_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample.yaml")
    }

    #[test]
    fn loads_fixture_entries() {
        let path = fixture_path();
        let entries = load_entries(&path).expect("failed to load fixture");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].message, "Investigated Service Bus retry behavior");
        assert!(!entries[0].decision);
        assert!(entries[1].decision);
    }

    #[test]
    fn saves_and_loads_entries() {
        let temp = TempDir::new().expect("temp dir");
        let path = temp.path().join("log.yaml");
        let entries = vec![LogEntry {
            timestamp: "2026-02-10T12:00:00Z".to_string(),
            message: "Test entry".to_string(),
            tags: vec!["test".to_string()],
            decision: false,
            git: GitContext {
                repo: "/repo".to_string(),
                branch: "main".to_string(),
                commit: "abc123".to_string(),
            },
        }];

        save_entries(&path, &entries).expect("save entries");
        let loaded = load_entries(&path).expect("load entries");
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].message, "Test entry");
    }
}
