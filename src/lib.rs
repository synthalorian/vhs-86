pub mod archive;
pub mod batch;
pub mod config;
pub mod disk_usage;
pub mod git;
pub mod keybindings;
pub mod permissions;
pub mod plugins;
pub mod preview;
pub mod profiling;
pub mod remote;
pub mod search;
pub mod theme;

use chrono::{DateTime, Local};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum EntryKind {
    Dir,
    File,
    Symlink,
    Unknown,
    Archive,
}

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub path: PathBuf,
    pub kind: EntryKind,
    pub size: u64,
    pub modified: Option<DateTime<Local>>,
}

pub fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "K", "M", "G", "T"];
    if size == 0 {
        return "-".to_string();
    }
    let mut size = size as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    if unit_idx == 0 {
        format!("{} {}", size as u64, UNITS[unit_idx])
    } else {
        format!("{:.1} {}", size, UNITS[unit_idx])
    }
}

pub fn format_time(dt: Option<DateTime<Local>>) -> String {
    dt.map(|d| d.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| "-".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size_zero() {
        assert_eq!(format_size(0), "-");
    }

    #[test]
    fn test_format_size_bytes() {
        assert_eq!(format_size(512), "512 B");
    }

    #[test]
    fn test_format_size_kilobytes() {
        assert_eq!(format_size(1536), "1.5 K");
    }

    #[test]
    fn test_format_size_megabytes() {
        assert_eq!(format_size(1024 * 1024), "1.0 M");
    }

    #[test]
    fn test_format_size_gigabytes() {
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 G");
    }

    #[test]
    fn test_format_size_terabytes() {
        assert_eq!(format_size(1024_u64 * 1024 * 1024 * 1024), "1.0 T");
    }

    #[test]
    fn test_format_time_none() {
        assert_eq!(format_time(None), "-");
    }

    #[test]
    fn test_format_time_some() {
        let dt = Local::now();
        let formatted = format_time(Some(dt));
        assert!(!formatted.is_empty());
        assert_ne!(formatted, "-");
    }
}
