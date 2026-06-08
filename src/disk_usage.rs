use std::fs;
use std::path::Path;

/// Calculate disk usage for a directory tree
#[derive(Debug, Clone)]
pub struct DiskEntry {
    pub name: String,
    pub path: std::path::PathBuf,
    pub size: u64,
    pub children: Vec<DiskEntry>,
    pub is_file: bool,
}

impl DiskEntry {
    pub fn total_size(&self) -> u64 {
        if self.is_file {
            self.size
        } else {
            self.children.iter().map(|c| c.total_size()).sum::<u64>() + self.size
        }
    }
}

/// Build a tree of disk usage starting from a path
pub fn build_disk_tree(path: &Path, max_depth: usize) -> Option<DiskEntry> {
    let name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("/")
        .to_string();

    let meta = fs::metadata(path).ok()?;

    if meta.is_file() {
        return Some(DiskEntry {
            name,
            path: path.to_path_buf(),
            size: meta.len(),
            children: Vec::new(),
            is_file: true,
        });
    }

    if meta.is_dir() && max_depth > 0 {
        let mut children = Vec::new();
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Some(child) = build_disk_tree(&entry.path(), max_depth - 1) {
                    children.push(child);
                }
            }
        }
        children.sort_by(|a, b| b.total_size().cmp(&a.total_size()));
        // Limit to top entries for display
        children.truncate(20);
        return Some(DiskEntry {
            name,
            path: path.to_path_buf(),
            size: 0,
            children,
            is_file: false,
        });
    }

    Some(DiskEntry {
        name,
        path: path.to_path_buf(),
        size: meta.len(),
        children: Vec::new(),
        is_file: false,
    })
}

/// Format a tree-map style visualization for terminal display
pub fn format_tree_map(entry: &DiskEntry, max_lines: usize, max_width: usize) -> Vec<(String, u8)> {
    let mut lines = Vec::new();
    let total = entry.total_size();
    if total == 0 {
        return lines;
    }

    lines.push((format!("{}: {}", entry.name, crate::format_size(total)), 0));

    let bar_width = max_width.saturating_sub(4);
    for (i, child) in entry.children.iter().take(max_lines.saturating_sub(1)).enumerate() {
        let child_total = child.total_size();
        let pct = child_total as f64 / total as f64;
        let filled = (bar_width as f64 * pct) as usize;
        let empty = bar_width.saturating_sub(filled);

        let bar = "█".repeat(filled) + &"░".repeat(empty);
        let line = format!("{:>2}. {:<20} {:>8} {:5.1}% {}",
            i + 1,
            truncate(&child.name, 20),
            crate::format_size(child_total),
            pct * 100.0,
            bar
        );
        lines.push((line, (pct * 100.0) as u8));
    }

    lines
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    } else {
        s.to_string()
    }
}

/// Quick disk usage analyzer that returns top-level directory sizes
pub fn analyze_directory(path: &Path) -> Vec<(String, u64)> {
    let mut sizes = Vec::new();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let size = calculate_size(&entry.path());
            sizes.push((name, size));
        }
    }
    sizes.sort_by(|a, b| b.1.cmp(&a.1));
    sizes
}

fn calculate_size(path: &Path) -> u64 {
    let meta = match fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return 0,
    };

    if meta.is_file() {
        meta.len()
    } else if meta.is_dir() {
        let mut total = 0u64;
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                total += calculate_size(&entry.path());
            }
        }
        total
    } else {
        0
    }
}

/// Disk usage view state
#[derive(Debug, Clone)]
pub struct DiskUsageView {
    pub visible: bool,
    pub path: Option<std::path::PathBuf>,
    pub entries: Vec<(String, u64)>,
    pub selected: usize,
}

impl DiskUsageView {
    pub fn new() -> Self {
        Self {
            visible: false,
            path: None,
            entries: Vec::new(),
            selected: 0,
        }
    }

    pub fn open(&mut self, path: &Path) {
        self.visible = true;
        self.path = Some(path.to_path_buf());
        self.entries = analyze_directory(path);
        self.selected = 0;
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.path = None;
        self.entries.clear();
        self.selected = 0;
    }

    pub fn move_down(&mut self) {
        if !self.entries.is_empty() {
            self.selected = (self.selected + 1).min(self.entries.len() - 1);
        }
    }

    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }
}
