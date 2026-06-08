use std::collections::HashMap;
use std::io::{self, Read};
use std::path::Path;

use crate::{DirEntry, EntryKind};

#[derive(Debug, Clone)]
pub enum ArchiveType {
    Zip,
    Tar,
    TarGz,
}

pub fn detect_archive(path: &Path) -> Option<ArchiveType> {
    let ext = path.extension().and_then(|e| e.to_str())?;
    match ext.to_lowercase().as_str() {
        "zip" => Some(ArchiveType::Zip),
        "tar" => Some(ArchiveType::Tar),
        "gz" => {
            let stem = path.file_stem().and_then(|s| s.to_str())?;
            if stem.ends_with(".tar") {
                Some(ArchiveType::TarGz)
            } else {
                None
            }
        }
        "tgz" => Some(ArchiveType::TarGz),
        _ => None,
    }
}

/// A virtual filesystem entry inside an archive
#[derive(Debug, Clone)]
pub struct ArchiveEntry {
    pub name: String,
    pub path: String,
    pub kind: EntryKind,
    pub size: u64,
    pub is_dir: bool,
}

/// Read the contents of a zip file without extracting
pub fn list_zip_entries(path: &Path) -> io::Result<Vec<ArchiveEntry>> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut entries = Vec::new();
    let mut seen_dirs = HashMap::new();

    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        let name = file.name().to_string();
        let is_dir = file.is_dir();
        let size = file.size();

        // Collect top-level entries only for browsing
        let parts: Vec<&str> = name.split('/').filter(|s| !s.is_empty()).collect();
        if parts.is_empty() {
            continue;
        }

        let top_name = parts[0].to_string();
        let top_path = if parts.len() > 1 || is_dir {
            format!("{}/", top_name)
        } else {
            top_name.clone()
        };

        if seen_dirs.contains_key(&top_name) {
            continue;
        }

        if parts.len() > 1 || is_dir {
            seen_dirs.insert(top_name.clone(), true);
            entries.push(ArchiveEntry {
                name: top_name,
                path: top_path,
                kind: EntryKind::Dir,
                size: 0,
                is_dir: true,
            });
        } else {
            entries.push(ArchiveEntry {
                name: top_name,
                path: top_path,
                kind: EntryKind::File,
                size,
                is_dir: false,
            });
        }
    }

    entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(entries)
}

/// Read the contents of a tar or tar.gz file without extracting
pub fn list_tar_entries(path: &Path, is_gz: bool) -> io::Result<Vec<ArchiveEntry>> {
    let file = std::fs::File::open(path)?;
    let mut entries = Vec::new();
    let mut seen_dirs = HashMap::new();

    let archive: Box<dyn Read> = if is_gz {
        Box::new(flate2::read::GzDecoder::new(file))
    } else {
        Box::new(file)
    };

    let mut tar = tar::Archive::new(archive);
    for entry in tar.entries()? {
        let entry = entry?;
        let path_bytes = entry.path_bytes();
        let name = String::from_utf8_lossy(&path_bytes).to_string();
        let is_dir = entry.header().entry_type().is_dir();
        let size = entry.header().size().unwrap_or(0);

        let parts: Vec<&str> = name.split('/').filter(|s| !s.is_empty()).collect();
        if parts.is_empty() {
            continue;
        }

        let top_name = parts[0].to_string();
        let top_path = if parts.len() > 1 || is_dir {
            format!("{}/", top_name)
        } else {
            top_name.clone()
        };

        if seen_dirs.contains_key(&top_name) {
            continue;
        }

        if parts.len() > 1 || is_dir {
            seen_dirs.insert(top_name.clone(), true);
            entries.push(ArchiveEntry {
                name: top_name,
                path: top_path,
                kind: EntryKind::Dir,
                size: 0,
                is_dir: true,
            });
        } else {
            entries.push(ArchiveEntry {
                name: top_name,
                path: top_path,
                kind: EntryKind::File,
                size,
                is_dir: false,
            });
        }
    }

    entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(entries)
}

/// Convert archive entries to DirEntry for display
pub fn archive_entries_to_direntries(
    archive_path: &Path,
    archive_entries: Vec<ArchiveEntry>,
) -> Vec<DirEntry> {
    archive_entries
        .into_iter()
        .map(|ae| DirEntry {
            name: ae.name,
            path: archive_path.join(&ae.path),
            kind: ae.kind,
            size: ae.size,
            modified: None,
        })
        .collect()
}

/// Get a friendly display name for archive type
pub fn archive_type_name(archive_type: &ArchiveType) -> &'static str {
    match archive_type {
        ArchiveType::Zip => "ZIP",
        ArchiveType::Tar => "TAR",
        ArchiveType::TarGz => "TAR.GZ",
    }
}
