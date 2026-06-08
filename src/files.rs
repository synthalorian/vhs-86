use std::fs;
use std::io;
use std::path::Path;
use walkdir::WalkDir;

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
