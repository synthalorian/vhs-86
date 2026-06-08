use std::fs;
use std::io::{self, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;

/// Check if a file is an image based on its extension.
pub fn is_image_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "tiff" | "tif" | "ico" | "avif"
            )
        })
        .unwrap_or(false)
}

/// Check if a file is a text file (for syntax highlighting vs hex view).
pub fn is_text_file(path: &Path) -> bool {
    if let Ok(mut file) = fs::File::open(path) {
        let mut buffer = [0u8; 1024];
        if let Ok(n) = file.read(&mut buffer) {
            return !buffer[..n].contains(&0);
        }
    }
    false
}

/// Generate a hex dump string for binary files.
pub fn hex_dump(path: &Path, max_bytes: usize) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut buffer = vec![0u8; max_bytes.min(4096)];
    let n = file.read(&mut buffer)?;
    buffer.truncate(n);

    let mut output = format!("Hex dump: {} ({} bytes)\n\n", path.display(), n);
    for (i, chunk) in buffer.chunks(16).enumerate() {
        let offset = i * 16;
        output.push_str(&format!("{:08x}  ", offset));
        for byte in chunk {
            output.push_str(&format!("{:02x} ", byte));
        }
        for _ in 0..(16 - chunk.len()) {
            output.push_str("   ");
        }
        output.push_str(" |");
        for byte in chunk {
            let c = if byte.is_ascii_graphic() || *byte == b' ' {
                *byte as char
            } else {
                '.'
            };
            output.push(c);
        }
        output.push_str("|\n");
    }
    Ok(output)
}

/// Create a zip archive from selected files/directories.
pub fn create_zip_archive(paths: &[PathBuf], output_path: &Path) -> io::Result<()> {
    let file = fs::File::create(output_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for path in paths {
        if path.is_file() {
            let name = path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            zip.start_file(name, options).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            let mut f = fs::File::open(path)?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            zip.write_all(&buf).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        } else if path.is_dir() {
            let base_name = path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            for entry in WalkDir::new(path).into_iter().flatten() {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    let rel_path = entry_path.strip_prefix(path)
                        .unwrap_or(entry_path)
                        .to_string_lossy()
                        .to_string();
                    let zip_name = format!("{}/{}", base_name, rel_path);
                    zip.start_file(zip_name, options).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                    let mut f = fs::File::open(entry_path)?;
                    let mut buf = Vec::new();
                    f.read_to_end(&mut buf)?;
                    zip.write_all(&buf).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                }
            }
        }
    }
    zip.finish().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    Ok(())
}

/// Create a tar.gz archive from selected files/directories.
pub fn create_tar_archive(paths: &[PathBuf], output_path: &Path) -> io::Result<()> {
    let file = fs::File::create(output_path)?;
    let gz = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    let mut tar = tar::Builder::new(gz);

    for path in paths {
        if path.is_file() {
            let mut f = fs::File::open(path)?;
            let name = path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            tar.append_file(name, &mut f)?;
        } else if path.is_dir() {
            tar.append_dir_all(
                path.file_name().unwrap_or(path.as_os_str()),
                path,
            )?;
        }
    }
    tar.finish()?;
    Ok(())
}

/// Extract a zip or tar.gz archive.
pub fn extract_archive(archive_path: &Path, output_dir: &Path) -> io::Result<()> {
    fs::create_dir_all(output_dir)?;
    let ext = archive_path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase());

    match ext.as_deref() {
        Some("zip") => {
            let file = fs::File::open(archive_path)?;
            let mut archive = zip::ZipArchive::new(BufReader::new(file))
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            for i in 0..archive.len() {
                let mut file = archive.by_index(i)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                let outpath = output_dir.join(file.name());
                if file.name().ends_with('/') {
                    fs::create_dir_all(&outpath)?;
                } else {
                    if let Some(parent) = outpath.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    let mut outfile = fs::File::create(&outpath)?;
                    io::copy(&mut file, &mut outfile)?;
                }
            }
        }
        Some("gz") | Some("tgz") => {
            let file = fs::File::open(archive_path)?;
            let gz = flate2::read::GzDecoder::new(file);
            let mut archive = tar::Archive::new(gz);
            archive.unpack(output_dir)?;
        }
        _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Unknown archive format")),
    }
    Ok(())
}

/// Check if file is an archive.
pub fn is_archive_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "zip" | "tar" | "gz" | "tgz"
            )
        })
        .unwrap_or(false)
}

/// Recursively copy a file or directory from `src` to `dst`.
/// `dst` is the full destination path (not the parent directory).
pub fn copy_item(src: &Path, dst: &Path) -> io::Result<()> {
    let meta = fs::metadata(src)?;
    if meta.is_dir() {
        copy_dir(src, dst)
    } else {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dst)?;
        Ok(())
    }
}

fn copy_dir(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in WalkDir::new(src).min_depth(1).into_iter().flatten() {
        let rel = entry.path().strip_prefix(src).unwrap_or(entry.path());
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

/// Move (rename) a file or directory from `src` to `dst`.
pub fn move_item(src: &Path, dst: &Path) -> io::Result<()> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(src, dst)
}

/// Recursively delete a file or directory.
pub fn delete_item(path: &Path) -> io::Result<()> {
    let meta = fs::metadata(path)?;
    if meta.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

/// Rename a file or directory within the same parent directory.
pub fn rename_item(dir: &Path, old_name: &str, new_name: &str) -> io::Result<()> {
    let src = dir.join(old_name);
    let dst = dir.join(new_name);
    fs::rename(src, dst)
}

/// Create a new directory.
pub fn create_dir(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)
}

/// Create a new empty file.
pub fn touch_file(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::OpenOptions::new().create(true).write(true).truncate(true).open(path)?;
    Ok(())
}

/// Open a file or directory with the system's default application.
pub fn open_item(path: &Path) -> io::Result<()> {
    open::that(path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

/// Batch rename selected files with a sequential pattern.
/// Pattern should contain a `{}` placeholder which will be replaced with the sequential number.
/// If the pattern contains `{:0N}` (e.g. `{:03}`), it will be zero-padded to N digits.
/// If no placeholder is found, the number is appended before the extension.
pub fn batch_rename(dir: &Path, items: &[(String, String)]) -> io::Result<(usize, usize)> {
    let mut success = 0;
    let mut failed = 0;
    for (old_name, new_name) in items {
        if let Err(_) = rename_item(dir, old_name, new_name) {
            failed += 1;
        } else {
            success += 1;
        }
    }
    Ok((success, failed))
}

/// Generate sequential filenames from a pattern and a list of source names.
/// Pattern examples:
///   "vacation_{:03}.jpg" → vacation_001.jpg, vacation_002.jpg, ...
///   "img_{}.png" → img_1.png, img_2.png, ...
///   "backup" → backup_1, backup_2, ... (number appended)
pub fn generate_sequential_names(pattern: &str, count: usize) -> Vec<String> {
    let mut names = Vec::with_capacity(count);

    // Check for {:0N} format
    let zero_padded_re = regex::Regex::new(r"\{:0(\d+)\}").ok();
    let has_brace = pattern.contains("{}");

    for i in 1..=count {
        let name = if let Some(ref re) = zero_padded_re {
            if let Some(caps) = re.captures(pattern) {
                let width: usize = caps[1].parse().unwrap_or(3);
                let num = format!("{:0width$}", i, width = width);
                re.replace(pattern, &num).to_string()
            } else if has_brace {
                pattern.replace("{}", &i.to_string())
            } else {
                format!("{}_{}", pattern, i)
            }
        } else if has_brace {
            pattern.replace("{}", &i.to_string())
        } else {
            format!("{}_{}", pattern, i)
        };
        names.push(name);
    }
    names
}

/// Format an error for display in the UI status line.
pub fn format_err(e: io::Error) -> String {
    format!("Error: {}", e)
}

/// Validate that a filename doesn't contain path separators or empty strings.
pub fn valid_name(name: &str) -> bool {
    !name.is_empty() && !name.contains('/') && !name.contains('\0')
}
