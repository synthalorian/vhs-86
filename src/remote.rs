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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ssh_target_with_user() {
        let result = parse_ssh_target("alice@example.com");
        assert!(result.is_some());
        let (user, host) = result.unwrap();
        assert_eq!(user, "alice");
        assert_eq!(host, "example.com");
    }

    #[test]
    fn test_parse_ssh_target_without_user() {
        let result = parse_ssh_target("example.com");
        if std::env::var("USER").is_ok() {
            assert!(result.is_some());
            let (_, host) = result.unwrap();
            assert_eq!(host, "example.com");
        } else {
            assert!(result.is_none());
        }
    }

    #[test]
    fn test_remote_fs_new() {
        let fs = RemoteFs::new();
        assert!(!fs.is_connected());
        assert!(fs.current_dir().is_none());
        assert!(fs.connection_string().is_none());
    }

    #[test]
    fn test_remote_fs_connect() {
        let mut fs = RemoteFs::new();
        assert!(fs.connect("example.com", "alice").is_ok());
        assert!(fs.is_connected());
        assert_eq!(fs.connection_string(), Some("alice@example.com".to_string()));
    }

    #[test]
    fn test_remote_fs_disconnect() {
        let mut fs = RemoteFs::new();
        fs.connect("example.com", "alice").unwrap();
        fs.disconnect();
        assert!(!fs.is_connected());
    }

    #[test]
    fn test_remote_fs_list_dir_disconnected() {
        let fs = RemoteFs::new();
        assert!(fs.list_dir(Path::new("/")).is_err());
    }

    #[test]
    fn test_remote_fs_list_dir_connected() {
        let mut fs = RemoteFs::new();
        fs.connect("example.com", "alice").unwrap();
        let entries = fs.list_dir(Path::new("/home/alice")).unwrap();
        assert!(!entries.is_empty());
    }
}
