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

#[derive(Debug, Display, Serialize)]
enum EntryType {
    File,
    Dir,
}

#[derive(Debug, Tabled, Serialize)]
struct FileEntry {
    #[tabled{rename="Name"}]
    name: String,
    #[tabled{rename="Type"}]
    e_type: EntryType,
    #[tabled{rename="Size B"}]
    len_bytes: u64,
    modified: String,
}

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
                        .unwrap_or("Cannot parse json".to_string())
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
            len_bytes: meta.len(),
            modified: if let Ok(modi) = meta.modified() {
                let date: DateTime<Utc> = modi.into();
                format!("{}", date.format("%a %b %e %Y"))
            } else {
                String::default()
            },
        })
    }
}
