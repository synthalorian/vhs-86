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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DirEntry, EntryKind};

    #[test]
    fn test_batch_selection_new() {
        let sel = BatchSelection::new();
        assert!(!sel.active);
        assert_eq!(sel.count(), 0);
    }

    #[test]
    fn test_batch_selection_toggle() {
        let mut sel = BatchSelection::new();
        sel.toggle(0);
        assert!(sel.active);
        assert_eq!(sel.count(), 1);
        assert!(sel.is_selected(0));

        sel.toggle(0);
        assert!(!sel.active);
        assert_eq!(sel.count(), 0);
        assert!(!sel.is_selected(0));
    }

    #[test]
    fn test_batch_selection_multiple() {
        let mut sel = BatchSelection::new();
        sel.toggle(0);
        sel.toggle(2);
        sel.toggle(5);
        assert_eq!(sel.count(), 3);
        assert!(sel.is_selected(0));
        assert!(!sel.is_selected(1));
        assert!(sel.is_selected(2));
    }

    #[test]
    fn test_batch_selection_clear() {
        let mut sel = BatchSelection::new();
        sel.toggle(0);
        sel.toggle(1);
        sel.clear();
        assert!(!sel.active);
        assert_eq!(sel.count(), 0);
    }

    #[test]
    fn test_batch_selection_get_selected_entries() {
        let mut sel = BatchSelection::new();
        let entries = vec![
            DirEntry {
                name: "a".to_string(),
                path: PathBuf::from("/a"),
                kind: EntryKind::File,
                size: 0,
                modified: None,
            },
            DirEntry {
                name: "b".to_string(),
                path: PathBuf::from("/b"),
                kind: EntryKind::File,
                size: 0,
                modified: None,
            },
        ];
        sel.toggle(0);
        let selected = sel.get_selected_entries(&entries);
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].name, "a");
    }

    #[test]
    fn test_batch_dialog_new() {
        let dialog = BatchDialog::new();
        assert!(!dialog.visible);
        assert!(dialog.action.is_none());
    }

    #[test]
    fn test_batch_dialog_open() {
        let mut dialog = BatchDialog::new();
        dialog.open(BatchActionType::Delete);
        assert!(dialog.visible);
        assert_eq!(dialog.action, Some(BatchActionType::Delete));
    }

    #[test]
    fn test_batch_dialog_action_name() {
        let mut dialog = BatchDialog::new();
        dialog.open(BatchActionType::Copy);
        assert_eq!(dialog.action_name(), "Copy to");
    }

    #[test]
    fn test_batch_dialog_input() {
        let mut dialog = BatchDialog::new();
        dialog.push_char('a');
        dialog.push_char('b');
        assert_eq!(dialog.input, "ab");
        dialog.pop_char();
        assert_eq!(dialog.input, "a");
    }

    #[test]
    fn test_delete_entry_file() {
        let tmpdir = tempfile::tempdir().unwrap();
        let file_path = tmpdir.path().join("test.txt");
        std::fs::write(&file_path, "hello").unwrap();
        assert!(file_path.exists());
        delete_entry(&file_path).unwrap();
        assert!(!file_path.exists());
    }

    #[test]
    fn test_delete_entry_dir() {
        let tmpdir = tempfile::tempdir().unwrap();
        let dir_path = tmpdir.path().join("subdir");
        std::fs::create_dir(&dir_path).unwrap();
        std::fs::write(dir_path.join("file.txt"), "hello").unwrap();
        delete_entry(&dir_path).unwrap();
        assert!(!dir_path.exists());
    }

    #[test]
    fn test_copy_entry_file() {
        let tmpdir = tempfile::tempdir().unwrap();
        let src = tmpdir.path().join("src.txt");
        let dest_dir = tmpdir.path().join("dest");
        std::fs::create_dir(&dest_dir).unwrap();
        std::fs::write(&src, "hello").unwrap();
        copy_entry(&src, &dest_dir).unwrap();
        assert!(dest_dir.join("src.txt").exists());
    }

    #[test]
    fn test_move_entry() {
        let tmpdir = tempfile::tempdir().unwrap();
        let src = tmpdir.path().join("src.txt");
        let dest_dir = tmpdir.path().join("dest");
        std::fs::create_dir(&dest_dir).unwrap();
        std::fs::write(&src, "hello").unwrap();
        move_entry(&src, &dest_dir).unwrap();
        assert!(!src.exists());
        assert!(dest_dir.join("src.txt").exists());
    }

    #[test]
    fn test_execute_batch_action_delete() {
        let tmpdir = tempfile::tempdir().unwrap();
        let file1 = tmpdir.path().join("a.txt");
        let file2 = tmpdir.path().join("b.txt");
        std::fs::write(&file1, "a").unwrap();
        std::fs::write(&file2, "b").unwrap();

        let entry1 = DirEntry {
            name: "a.txt".to_string(),
            path: file1.clone(),
            kind: EntryKind::File,
            size: 1,
            modified: None,
        };
        let entry2 = DirEntry {
            name: "b.txt".to_string(),
            path: file2.clone(),
            kind: EntryKind::File,
            size: 1,
            modified: None,
        };
        let entries: Vec<&DirEntry> = vec![&entry1, &entry2];

        let action = BatchAction::Delete;
        let (success, failed) = execute_batch_action(&action, &entries).unwrap();
        assert_eq!(success, 2);
        assert_eq!(failed, 0);
        assert!(!file1.exists());
        assert!(!file2.exists());
    }
}
