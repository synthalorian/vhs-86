use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitStatus {
    Added,
    Modified,
    Untracked,
    Unchanged,
}

pub struct GitCache {
    repo_path: Option<PathBuf>,
    statuses: HashMap<PathBuf, GitStatus>,
}

impl GitCache {
    pub fn new() -> Self {
        Self {
            repo_path: None,
            statuses: HashMap::new(),
        }
    }

    pub fn refresh(&mut self, cwd: &Path) {
        self.statuses.clear();
        self.repo_path = find_git_root(cwd);

        if let Some(ref repo_path) = self.repo_path {
            if let Ok(repo) = git2::Repository::open(repo_path) {
                if let Ok(statuses) = repo.statuses(None) {
                    for entry in statuses.iter() {
                        if let Some(path) = entry.path() {
                            let full_path = repo_path.join(path);
                            let status = map_status(entry.status());
                            self.statuses.insert(full_path, status);
                        }
                    }
                }
            }
        }
    }

    pub fn get_status(&self, path: &Path) -> GitStatus {
        self.statuses.get(path).copied().unwrap_or(GitStatus::Unchanged)
    }

    pub fn is_repo(&self) -> bool {
        self.repo_path.is_some()
    }
}

fn find_git_root(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(path) = current {
        if path.join(".git").exists() {
            return Some(path.to_path_buf());
        }
        current = path.parent();
    }
    None
}

fn map_status(status: git2::Status) -> GitStatus {
    if status.contains(git2::Status::INDEX_NEW) || status.contains(git2::Status::WT_NEW) {
        GitStatus::Added
    } else if status.contains(git2::Status::INDEX_MODIFIED)
        || status.contains(git2::Status::WT_MODIFIED)
        || status.contains(git2::Status::INDEX_RENAMED)
    {
        GitStatus::Modified
    } else if status.contains(git2::Status::WT_NEW) {
        GitStatus::Untracked
    } else {
        GitStatus::Unchanged
    }
}
