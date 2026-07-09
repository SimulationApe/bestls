use chrono::{DateTime, Utc};
use clap::Parser;
use owo_colors::OwoColorize;
use serde::Serialize;
use strum::Display;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tabled::{
    Table, 
    Tabled, 
    settings::{
        Style, 
        Color, 
        object:: {Columns, Rows}
    },
};

/// Whether a filesystem entry is a file or directory.
#[derive(Debug, Display, Serialize)]
enum EntryType {
    File,
    Dir,
}

/// A filesystem entry displayed in the output table.
///
/// Columns: `Name` (entry name), `Type` (File/Dir), `Size` (human-readable),
/// `Modified` (relative timestamp).
#[derive(Debug, Tabled, Serialize)]
struct FileEntry {
    #[tabled{rename="Name"}]
    name: String,
    #[tabled{rename="Type"}]
    e_type: EntryType,
    #[tabled{rename="Size"}]
    size: String,
    #[tabled{rename="Modified"}]
    modified: String,
}

/// Command-line interface for bestls.
///
/// Accepts an optional directory path and a `--json` flag for
/// machine-readable JSON output instead of the default colored table.
#[derive(Debug, Parser)]
#[command(version, about, long_about = "Best Ls command ever")]
struct CLI {
    path: Option<PathBuf>,
    #[arg(short, long)]
    json: bool,
}

fn main() {
    let cli = CLI::parse();

    let path = cli.path.unwrap_or(PathBuf::from("."));

    if let Ok(does_exist) = fs::exists(&path) {
        if does_exist {
            if cli.json {
                let entries = get_files(&path);
                println!(
                    "{}",
                    serde_json::to_string(&entries)
                        .unwrap_or_else(|e| format!("Cannot serialize to JSON: {e}"))
                );
            } else {
                print_table(&path);
            }
        } else {
            println!("{}", "Path does not exist".red());
        }
    } else {
        println!("{}", "Error reading directory".red());
    }
}

fn print_table(path: &Path) {
    let get_files = get_files(path);
    let mut table = Table::new(get_files);
    table.with(Style::rounded());
    table.modify(Columns::first(), Color::FG_BRIGHT_CYAN);
    table.modify(Columns::one(2), Color::FG_BRIGHT_MAGENTA);
    table.modify(Columns::one(3), Color::FG_BRIGHT_YELLOW);
    table.modify(Rows::first(), Color::FG_BRIGHT_GREEN);
    println!("{}", table);
}

fn get_files(path: &Path) -> Vec<FileEntry> {
    let mut data = Vec::default();
    if let Ok(read_dir) = fs::read_dir(path) {
        for entry in read_dir {
            if let Ok(file) = entry {
                map_data(file, &mut data);
            }
        }
    }
    data
}

fn map_data(file: fs::DirEntry, data: &mut Vec<FileEntry>) {
    if let Ok(meta) = fs::metadata(&file.path()) {
        data.push(FileEntry {
            name: file
                .file_name()
                .into_string()
                .unwrap_or("unknown name".into()),
            e_type: if meta.is_dir() {
                EntryType::Dir
            } else {
                EntryType::File
            },
            size: format_size(meta.len()),
            modified: if let Ok(modi) = meta.modified() {
                let date: DateTime<Utc> = modi.into();
                format_relative_time(date)
            } else {
                String::default()
            },
        })
    } else {
        eprintln!(
            "Warning: cannot read metadata for '{}', skipping",
            file.path().display()
        );
    }
}

/// Format a byte count as a human-readable string.
///
/// Bytes are displayed as whole numbers (e.g. `42 B`).
/// Values ≥ 1 KB are displayed with 1 decimal place (e.g. `1.5 KB`).
/// Supports up to terabytes (TB).
fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{} {}", bytes, UNITS[unit_idx])
    } else {
        format!("{:.1} {}", size, UNITS[unit_idx])
    }
}

/// Format a UTC timestamp as a relative time phrase.
///
/// Returns phrases like `just now`, `3 minutes ago`, `2 days ago`,
/// `1 month ago`, or `5 years ago` based on the elapsed time since `date`.
fn format_relative_time(date: DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(date);

    if duration.num_seconds() < 0 {
        return String::from("in the future");
    }

    let secs = duration.num_seconds();
    if secs < 60 {
        return String::from("just now");
    }

    let mins = duration.num_minutes();
    if mins < 60 {
        return format!("{} minute{} ago", mins, if mins == 1 { "" } else { "s" });
    }

    let hours = duration.num_hours();
    if hours < 24 {
        return format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" });
    }

    let days = duration.num_days();
    if days < 30 {
        return format!("{} day{} ago", days, if days == 1 { "" } else { "s" });
    }

    if days < 365 {
        let months = days / 30;
        return format!("{} month{} ago", months, if months == 1 { "" } else { "s" });
    }

    let years = days / 365;
    format!("{} year{} ago", years, if years == 1 { "" } else { "s" })
}
