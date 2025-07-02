use clap::Parser;
use owo_colors::OwoColorize;
use std::{
    fs,
    string::String,
    path::{Path, PathBuf},
};

#[derive(Debug, Parser)]
#[command(version, about, long_about = "Best Ls command ever")]
struct CLI {
    path: Option<PathBuf>,
}

fn main() {
    let cli = CLI::parse();

    let path = cli.path.unwrap_or(PathBuf::from("."));

    if let Ok(does_exist) = fs::exists(&path) {
        if does_exist {
            for file in get_files(&path) {
                println!("{}", file);
            }
        } else {
            println!("{}", "Path does not exist".red());
        }
    } else {
        println!("{}", "Error reading directory".red());
    }
}

fn get_files(path: &Path) -> Vec<String> {
    let mut data = Vec::default();
    if let Ok(read_dir) = fs::read_dir(path) {
        for entry in read_dir {
            if let Ok(file) = entry {
                data.push(
                    file.file_name()
                        .into_string()
                        .unwrap_or("unknown name".into())
                );
            }
        }
    }
    data
}
