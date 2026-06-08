use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::DirEntry;

/// Manages batch file selections and operations
#[derive(Debug, Clone)]
pub struct BatchSelection {
    pub selected_indices: HashSet<usize>,
    pub active: bool,
}

impl BatchSelection {
    pub fn new() -> Self {
        Self {
            selected_indices: HashSet::new(),
            active: false,
        }
    }

    pub fn toggle(&mut self, index: usize) {
        if self.selected_indices.contains(&index) {
            self.selected_indices.remove(&index);
        } else {
            self.selected_indices.insert(index);
        }
        if !self.selected_indices.is_empty() {
            self.active = true;
        } else {
            self.active = false;
        }
    }

    pub fn is_selected(&self, index: usize) -> bool {
        self.selected_indices.contains(&index)
    }

    pub fn clear(&mut self) {
        self.selected_indices.clear();
        self.active = false;
    }

    pub fn count(&self) -> usize {
        self.selected_indices.len()
    }

    pub fn get_selected_entries<'a>(&self, entries: &'a [DirEntry]) -> Vec<&'a DirEntry> {
        self.selected_indices
            .iter()
            .filter_map(|&idx| entries.get(idx))
            .collect()
    }
}

/// Batch operations that can be applied to selected files
#[derive(Debug, Clone)]
pub enum BatchAction {
    Delete,
    Copy { dest: PathBuf },
    Move { dest: PathBuf },
}

/// Execute a batch action on selected entries
pub fn execute_batch_action(
    action: &BatchAction,
    entries: &[&DirEntry],
) -> Result<(usize, usize), String> {
    let mut success = 0usize;
    let mut failed = 0usize;

    for entry in entries {
        match action {
            BatchAction::Delete => {
                match delete_entry(&entry.path) {
                    Ok(_) => success += 1,
                    Err(_) => failed += 1,
                }
            }
            BatchAction::Copy { dest } => {
                match copy_entry(&entry.path, dest) {
                    Ok(_) => success += 1,
                    Err(_) => failed += 1,
                }
            }
            BatchAction::Move { dest } => {
                match move_entry(&entry.path, dest) {
                    Ok(_) => success += 1,
                    Err(_) => failed += 1,
                }
            }
        }
    }

    Ok((success, failed))
}

fn delete_entry(path: &Path) -> io::Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn copy_entry(src: &Path, dest_dir: &Path) -> io::Result<()> {
    let file_name = src.file_name().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "Invalid source path")
    })?;
    let dest = dest_dir.join(file_name);

    if src.is_dir() {
        copy_dir_all(src, &dest)?;
    } else {
        fs::copy(src, dest)?;
    }
    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn move_entry(src: &Path, dest_dir: &Path) -> io::Result<()> {
    let file_name = src.file_name().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "Invalid source path")
    })?;
    let dest = dest_dir.join(file_name);
    fs::rename(src, dest)?;
    Ok(())
}

/// Batch action dialog state
#[derive(Debug, Clone)]
pub struct BatchDialog {
    pub visible: bool,
    pub action: Option<BatchActionType>,
    pub input: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BatchActionType {
    Delete,
    Copy,
    Move,
}

impl BatchDialog {
    pub fn new() -> Self {
        Self {
            visible: false,
            action: None,
            input: String::new(),
        }
    }

    pub fn open(&mut self, action: BatchActionType) {
        self.visible = true;
        self.action = Some(action);
        self.input.clear();
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.action = None;
        self.input.clear();
    }

    pub fn push_char(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn pop_char(&mut self) {
        self.input.pop();
    }

    pub fn action_name(&self) -> &'static str {
        match self.action {
            Some(BatchActionType::Delete) => "Delete",
            Some(BatchActionType::Copy) => "Copy to",
            Some(BatchActionType::Move) => "Move to",
            None => "",
        }
    }
}
