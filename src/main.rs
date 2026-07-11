use chrono::{DateTime, Utc};
use clap::Parser;
use owo_colors::OwoColorize;
use serde::Serialize;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
use tabled::{
    Table, Tabled,
    settings::{
        Color, Style,
        object::{Columns, Rows},
    },
};

/// File type classification with emoji for display badges.
#[derive(Debug, Clone, Copy)]
enum EntryType {
    File,
    Dir,
    Symlink,
    Pipe,
    Socket,
    BlockDevice,
    CharDevice,
}

impl EntryType {
    /// Detect file type from Unix mode bits.
    fn from_mode(mode: u32) -> Self {
        match mode & libc::S_IFMT {
            libc::S_IFDIR => Self::Dir,
            libc::S_IFLNK => Self::Symlink,
            libc::S_IFIFO => Self::Pipe,
            libc::S_IFSOCK => Self::Socket,
            libc::S_IFBLK => Self::BlockDevice,
            libc::S_IFCHR => Self::CharDevice,
            _ => Self::File,
        }
    }

    /// Plain display name (used in JSON output).
    fn name(&self) -> &str {
        match self {
            Self::File => "File",
            Self::Dir => "Dir",
            Self::Symlink => "Symlink",
            Self::Pipe => "Pipe",
            Self::Socket => "Socket",
            Self::BlockDevice => "Block Device",
            Self::CharDevice => "Char Device",
        }
    }

    /// Colored badge string for table display: emoji + name with color.
    fn format_badge(&self) -> String {
        let emoji = match self {
            Self::File => "\u{1f4c4}",
            Self::Dir => "\u{1f4c1}",
            Self::Symlink => "\u{1f517}",
            Self::Pipe => "\u{1f4e1}",
            Self::Socket => "\u{1f50c}",
            Self::BlockDevice => "\u{1f4be}",
            Self::CharDevice => "\u{2328}\u{fe0f}",
        };

        let label: String = match self {
            Self::File => "File".bright_white().to_string(),
            Self::Dir => "Dir".bright_blue().to_string(),
            Self::Symlink => "Symlink".bright_cyan().to_string(),
            Self::Pipe => "Pipe".yellow().to_string(),
            Self::Socket => "Socket".bright_green().to_string(),
            Self::BlockDevice => "Block".bright_magenta().to_string(),
            Self::CharDevice => "Char".bright_magenta().to_string(),
        };

        format!("{} {}", emoji, label)
    }
}

/// Raw filesystem entry data (before formatting for any output mode).
#[derive(Debug)]
struct RawEntry {
    name: String,
    file_type: EntryType,
    mode: u32,
    size: u64,
    modified: String,
}

/// A filesystem entry for table display with colored badges and visual permissions.
#[derive(Debug, Tabled)]
struct FileEntry {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Type")]
    entry_type: String,
    #[tabled(rename = "Permissions")]
    permissions: String,
    #[tabled(rename = "Size")]
    size: String,
    #[tabled(rename = "Modified")]
    modified: String,
}

/// A filesystem entry for JSON output (plain values, no ANSI).
#[derive(Debug, Serialize)]
struct FileEntryJson {
    name: String,
    #[serde(rename = "type")]
    entry_type: String,
    permissions: String,
    size: String,
    modified: String,
}

/// Command-line interface for bestls.
#[derive(Debug, Parser)]
#[command(version, about, long_about = "Best Ls command ever")]
struct Cli {
    path: Option<PathBuf>,
    #[arg(short, long)]
    json: bool,
}

fn main() {
    let cli = Cli::parse();
    let path = cli.path.unwrap_or(PathBuf::from("."));

    if let Ok(does_exist) = fs::exists(&path) {
        if does_exist {
            let raw_entries = get_files(&path);
            if cli.json {
                let entries: Vec<FileEntryJson> = raw_entries
                    .iter()
                    .map(|r| to_file_entry_json(r))
                    .collect();
                println!(
                    "{}",
                    serde_json::to_string(&entries)
                        .unwrap_or_else(|e| format!("Cannot serialize to JSON: {e}"))
                );
            } else {
                let entries: Vec<FileEntry> = raw_entries
                    .iter()
                    .map(|r| to_file_entry(r))
                    .collect();
                print_table(entries);
            }
        } else {
            println!("{}", "Path does not exist".red());
        }
    } else {
        println!("{}", "Error reading directory".red());
    }
}

fn print_table(entries: Vec<FileEntry>) {
    let mut table = Table::new(entries);
    table.with(Style::rounded());
    table.modify(Columns::first(), Color::FG_BRIGHT_CYAN);
    table.modify(Columns::one(3), Color::FG_BRIGHT_YELLOW);
    table.modify(Rows::first(), Color::FG_BRIGHT_GREEN);
    println!("{}", table);
}

fn get_files(path: &Path) -> Vec<RawEntry> {
    let mut data = Vec::new();
    if let Ok(read_dir) = fs::read_dir(path) {
        for file in read_dir.flatten() {
            if let Some(entry) = map_data(file) {
                data.push(entry);
            }
        }
    }
    data
}

fn map_data(file: fs::DirEntry) -> Option<RawEntry> {
    // Check if this is a symlink first -- metadata() follows symlinks.
    let is_symlink = file.file_type().ok().map(|ft| ft.is_symlink()).unwrap_or(false);

    // Use metadata() for the target (for size/date), but symlink_metadata()
    // for the symlink itself (for permissions and type detection).
    let meta = match fs::metadata(file.path()) {
        Ok(m) => m,
        Err(_) => {
            eprintln!(
                "Warning: cannot read metadata for '{}', skipping",
                file.path().display()
            );
            return None;
        }
    };

    let (mode, file_type) = if is_symlink {
        // symlink_metadata gives us the symlink's own mode, not the target's.
        let sym_mode = fs::symlink_metadata(file.path())
            .ok()
            .map(|m| m.permissions().mode())
            .unwrap_or(libc::S_IFLNK | 0o777);
        (sym_mode, EntryType::Symlink)
    } else {
        let m = meta.permissions().mode();
        (m, EntryType::from_mode(m))
    };
    let size = meta.len();
    let modified = if let Ok(modi) = meta.modified() {
        let date: DateTime<Utc> = modi.into();
        format_relative_time(date)
    } else {
        String::default()
    };

    let name = file
        .file_name()
        .into_string()
        .unwrap_or_else(|_| "unknown name".into());

    Some(RawEntry {
        name,
        file_type,
        mode,
        size,
        modified,
    })
}

fn to_file_entry(raw: &RawEntry) -> FileEntry {
    FileEntry {
        name: raw.name.clone(),
        entry_type: raw.file_type.format_badge(),
        permissions: format_permissions_visual(raw.mode),
        size: format_size(raw.size),
        modified: raw.modified.clone(),
    }
}

fn to_file_entry_json(raw: &RawEntry) -> FileEntryJson {
    FileEntryJson {
        name: raw.name.clone(),
        entry_type: raw.file_type.name().to_string(),
        permissions: format_permissions_traditional(raw.mode),
        size: format_size(raw.size),
        modified: raw.modified.clone(),
    }
}

/// Format a byte count as a human-readable string.
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

/// Format Unix permission mode bits as a visual color-coded string.
///
/// Returns strings like `[rwx][r-x][r--]` where each r/w/x character
/// is colored green when present and red dash when absent. Each permission
/// group (owner/group/other) is wrapped in brackets.
fn format_permissions_visual(mode: u32) -> String {
    let perm_bit = |mask: u32, ch: char| -> String {
        if mode & mask != 0 {
            ch.green().to_string()
        } else {
            '-'.red().to_string()
        }
    };

    let owner = format!(
        "{}{}{}",
        perm_bit(libc::S_IRUSR, 'r'),
        perm_bit(libc::S_IWUSR, 'w'),
        perm_bit(libc::S_IXUSR, 'x'),
    );
    let group = format!(
        "{}{}{}",
        perm_bit(libc::S_IRGRP, 'r'),
        perm_bit(libc::S_IWGRP, 'w'),
        perm_bit(libc::S_IXGRP, 'x'),
    );
    let other = format!(
        "{}{}{}",
        perm_bit(libc::S_IROTH, 'r'),
        perm_bit(libc::S_IWOTH, 'w'),
        perm_bit(libc::S_IXOTH, 'x'),
    );

    format!("[{}][{}][{}]", owner, group, other)
}

/// Format Unix permission mode bits as a traditional 10-character string.
///
/// Used for JSON output. Returns strings like `-rw-r--r--` or `drwxr-xr-x`.
fn format_permissions_traditional(mode: u32) -> String {
    let file_type = match mode & libc::S_IFMT {
        libc::S_IFDIR => 'd',
        libc::S_IFLNK => 'l',
        libc::S_IFIFO => 'p',
        libc::S_IFSOCK => 's',
        libc::S_IFBLK => 'b',
        libc::S_IFCHR => 'c',
        _ => '-',
    };

    let perm_bit = |mask: u32, ch: char| -> char {
        if mode & mask != 0 { ch } else { '-' }
    };

    format!(
        "{}{}{}{}{}{}{}{}{}{}",
        file_type,
        perm_bit(libc::S_IRUSR, 'r'),
        perm_bit(libc::S_IWUSR, 'w'),
        perm_bit(libc::S_IXUSR, 'x'),
        perm_bit(libc::S_IRGRP, 'r'),
        perm_bit(libc::S_IWGRP, 'w'),
        perm_bit(libc::S_IXGRP, 'x'),
        perm_bit(libc::S_IROTH, 'r'),
        perm_bit(libc::S_IWOTH, 'w'),
        perm_bit(libc::S_IXOTH, 'x'),
    )
}

/// Format a UTC timestamp as a relative time phrase.
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
