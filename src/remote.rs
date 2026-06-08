use std::io;
use std::path::{Path, PathBuf};

use crate::{DirEntry, EntryKind};

/// Represents a remote filesystem connection
#[derive(Debug)]
pub enum RemoteFs {
    Disconnected,
    Connected(RemoteConnection),
}

#[derive(Debug, Clone)]
pub struct RemoteConnection {
    pub host: String,
    pub user: String,
    pub current_dir: PathBuf,
}

impl RemoteFs {
    pub fn new() -> Self {
        RemoteFs::Disconnected
    }

    pub fn is_connected(&self) -> bool {
        matches!(self, RemoteFs::Connected(_))
    }

    pub fn connect(&mut self, host: &str, user: &str) -> io::Result<()> {
        // Simplified connection - in full implementation this would use russh
        // For now, we store the connection info and simulate basic functionality
        *self = RemoteFs::Connected(RemoteConnection {
            host: host.to_string(),
            user: user.to_string(),
            current_dir: PathBuf::from("/home/").join(user),
        });
        Ok(())
    }

    pub fn disconnect(&mut self) {
        *self = RemoteFs::Disconnected;
    }

    pub fn list_dir(&self, path: &Path) -> io::Result<Vec<DirEntry>> {
        match self {
            RemoteFs::Disconnected => Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Not connected to remote host",
            )),
            RemoteFs::Connected(conn) => {
                // Simulated remote listing - in full implementation this would use SFTP
                // For demonstration, we show the connection info as virtual entries
                let mut entries = Vec::new();
                entries.push(DirEntry {
                    name: "..".to_string(),
                    path: path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| path.to_path_buf()),
                    kind: EntryKind::Dir,
                    size: 0,
                    modified: None,
                });
                entries.push(DirEntry {
                    name: format!("[remote: {}@{}]", conn.user, conn.host),
                    path: path.to_path_buf(),
                    kind: EntryKind::Dir,
                    size: 0,
                    modified: None,
                });
                Ok(entries)
            }
        }
    }

    pub fn current_dir(&self) -> Option<PathBuf> {
        match self {
            RemoteFs::Connected(conn) => Some(conn.current_dir.clone()),
            _ => None,
        }
    }

    pub fn connection_string(&self) -> Option<String> {
        match self {
            RemoteFs::Connected(conn) => Some(format!("{}@{}", conn.user, conn.host)),
            _ => None,
        }
    }
}

/// Parse an SSH target string like "user@host" or "host"
pub fn parse_ssh_target(target: &str) -> Option<(String, String)> {
    if let Some((user, host)) = target.split_once('@') {
        Some((user.to_string(), host.to_string()))
    } else {
        // If no user specified, try to use current user
        if let Ok(user) = std::env::var("USER") {
            Some((user, target.to_string()))
        } else {
            None
        }
    }
}
