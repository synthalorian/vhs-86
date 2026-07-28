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
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            zip.start_file(name, options).map_err(io::Error::other)?;
            let mut f = fs::File::open(path)?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            zip.write_all(&buf).map_err(io::Error::other)?;
        } else if path.is_dir() {
            let base_name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            for entry in WalkDir::new(path).into_iter().flatten() {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    let rel_path = entry_path
                        .strip_prefix(path)
                        .unwrap_or(entry_path)
                        .to_string_lossy()
                        .to_string();
                    let zip_name = format!("{}/{}", base_name, rel_path);
                    zip.start_file(zip_name, options)
                        .map_err(io::Error::other)?;
                    let mut f = fs::File::open(entry_path)?;
                    let mut buf = Vec::new();
                    f.read_to_end(&mut buf)?;
                    zip.write_all(&buf).map_err(io::Error::other)?;
                }
            }
        }
    }
    zip.finish().map_err(io::Error::other)?;
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
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            tar.append_file(name, &mut f)?;
        } else if path.is_dir() {
            tar.append_dir_all(path.file_name().unwrap_or(path.as_os_str()), path)?;
        }
    }
    tar.finish()?;
    Ok(())
}

/// Extract a zip or tar.gz archive.
pub fn extract_archive(archive_path: &Path, output_dir: &Path) -> io::Result<()> {
    fs::create_dir_all(output_dir)?;
    let ext = archive_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase());

    match ext.as_deref() {
        Some("zip") => {
            let file = fs::File::open(archive_path)?;
            let mut archive =
                zip::ZipArchive::new(BufReader::new(file)).map_err(io::Error::other)?;
            for i in 0..archive.len() {
                let mut file = archive.by_index(i).map_err(io::Error::other)?;
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
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unknown archive format",
            ));
        }
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
    fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;
    Ok(())
}

/// Open a file or directory with the system's default application.
pub fn open_item(path: &Path) -> io::Result<()> {
    open::that(path).map_err(io::Error::other)
}

/// Batch rename selected files with a sequential pattern.
/// Pattern should contain a `{}` placeholder which will be replaced with the sequential number.
/// If the pattern contains `{:0N}` (e.g. `{:03}`), it will be zero-padded to N digits.
/// If no placeholder is found, the number is appended before the extension.
pub fn batch_rename(dir: &Path, items: &[(String, String)]) -> io::Result<(usize, usize)> {
    let mut success = 0;
    let mut failed = 0;
    for (old_name, new_name) in items {
        if rename_item(dir, old_name, new_name).is_err() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_file(dir: &Path, name: &str, contents: &[u8]) -> PathBuf {
        let p = dir.join(name);
        fs::write(&p, contents).unwrap();
        p
    }

    // --- is_image_file ---

    #[test]
    fn image_extensions_detected_case_insensitively() {
        for name in [
            "a.png", "b.JPG", "c.jpeg", "d.gif", "e.bmp", "f.webp", "g.tiff", "h.avif",
        ] {
            assert!(
                is_image_file(Path::new(name)),
                "{} should be an image",
                name
            );
        }
    }

    #[test]
    fn non_image_extensions_rejected() {
        for name in ["a.txt", "b.rs", "c.zip", "noext", ".hidden"] {
            assert!(
                !is_image_file(Path::new(name)),
                "{} should not be an image",
                name
            );
        }
    }

    // --- is_archive_file ---

    #[test]
    fn archive_extensions_detected() {
        for name in ["a.zip", "b.tar", "c.gz", "d.tgz", "e.ZIP"] {
            assert!(
                is_archive_file(Path::new(name)),
                "{} should be an archive",
                name
            );
        }
        assert!(!is_archive_file(Path::new("f.png")));
        assert!(!is_archive_file(Path::new("noext")));
    }

    // --- is_text_file ---

    #[test]
    fn text_file_detected_as_text() {
        let tmp = TempDir::new().unwrap();
        let p = write_file(tmp.path(), "hello.txt", b"hello world\nthis is text\n");
        assert!(is_text_file(&p));
    }

    #[test]
    fn binary_file_with_nul_detected_as_binary() {
        let tmp = TempDir::new().unwrap();
        let p = write_file(tmp.path(), "bin.dat", &[0x89, 0x50, 0x00, 0xFF, 0x01]);
        assert!(!is_text_file(&p));
    }

    // --- hex_dump ---

    #[test]
    fn hex_dump_formats_offsets_bytes_and_ascii() {
        let tmp = TempDir::new().unwrap();
        let p = write_file(tmp.path(), "data.bin", b"AB");
        let dump = hex_dump(&p, 2048).unwrap();
        assert!(dump.contains("00000000"), "missing offset: {}", dump);
        assert!(dump.contains("41 42"), "missing hex bytes: {}", dump);
        assert!(dump.contains("|AB|"), "missing ascii column: {}", dump);
    }

    #[test]
    fn hex_dump_replaces_non_printable_with_dot() {
        let tmp = TempDir::new().unwrap();
        let p = write_file(tmp.path(), "data.bin", &[0x00, 0x41]);
        let dump = hex_dump(&p, 2048).unwrap();
        assert!(
            dump.contains("|.A|"),
            "non-printable should be '.': {}",
            dump
        );
    }

    // --- zip archive round trip ---

    #[test]
    fn zip_create_list_and_extract_round_trip() {
        let tmp = TempDir::new().unwrap();
        let src_dir = tmp.path().join("srcdir");
        fs::create_dir(&src_dir).unwrap();
        write_file(&src_dir, "one.txt", b"first");
        let sub = src_dir.join("sub");
        fs::create_dir(&sub).unwrap();
        write_file(&sub, "two.txt", b"second");

        let zip_path = tmp.path().join("out.zip");
        create_zip_archive(&[src_dir], &zip_path).unwrap();
        assert!(is_archive_file(&zip_path));

        // Listing: verify entry names inside the archive
        let file = fs::File::open(&zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(BufReader::new(file)).unwrap();
        let names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();
        assert!(
            names.contains(&"srcdir/one.txt".to_string()),
            "names: {:?}",
            names
        );
        assert!(
            names.contains(&"srcdir/sub/two.txt".to_string()),
            "names: {:?}",
            names
        );

        // Extract and verify contents
        let out_dir = tmp.path().join("extracted");
        extract_archive(&zip_path, &out_dir).unwrap();
        assert_eq!(fs::read(out_dir.join("srcdir/one.txt")).unwrap(), b"first");
        assert_eq!(
            fs::read(out_dir.join("srcdir/sub/two.txt")).unwrap(),
            b"second"
        );
    }

    // --- tar.gz archive round trip ---

    #[test]
    fn tar_create_list_and_extract_round_trip() {
        let tmp = TempDir::new().unwrap();
        let src_dir = tmp.path().join("srcdir");
        fs::create_dir(&src_dir).unwrap();
        write_file(&src_dir, "alpha.txt", b"aaa");
        let sub = src_dir.join("nested");
        fs::create_dir(&sub).unwrap();
        write_file(&sub, "beta.txt", b"bbb");

        let tar_path = tmp.path().join("out.tar.gz");
        create_tar_archive(&[src_dir], &tar_path).unwrap();
        assert!(is_archive_file(&tar_path));

        // Listing: verify entry names inside the tar
        let file = fs::File::open(&tar_path).unwrap();
        let gz = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(gz);
        let names: Vec<String> = archive
            .entries()
            .unwrap()
            .map(|e| {
                let e = e.unwrap();
                e.path().unwrap().to_string_lossy().to_string()
            })
            .collect();
        assert!(
            names.iter().any(|n| n.ends_with("alpha.txt")),
            "names: {:?}",
            names
        );
        assert!(
            names.iter().any(|n| n.ends_with("nested/beta.txt")),
            "names: {:?}",
            names
        );

        // Extract and verify contents
        let out_dir = tmp.path().join("extracted");
        extract_archive(&tar_path, &out_dir).unwrap();
        assert_eq!(fs::read(out_dir.join("srcdir/alpha.txt")).unwrap(), b"aaa");
        assert_eq!(
            fs::read(out_dir.join("srcdir/nested/beta.txt")).unwrap(),
            b"bbb"
        );
    }

    #[test]
    fn extract_unknown_format_errors() {
        let tmp = TempDir::new().unwrap();
        let p = write_file(tmp.path(), "file.rar", b"nope");
        let err = extract_archive(&p, &tmp.path().join("out")).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }

    // --- copy / move / delete / rename ---

    #[test]
    fn copy_item_copies_file_contents() {
        let tmp = TempDir::new().unwrap();
        let src = write_file(tmp.path(), "src.txt", b"payload");
        let dst = tmp.path().join("dst.txt");
        copy_item(&src, &dst).unwrap();
        assert_eq!(fs::read(&dst).unwrap(), b"payload");
        assert!(src.exists(), "copy must leave source intact");
    }

    #[test]
    fn copy_item_recurses_directories() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("dir");
        fs::create_dir(&src).unwrap();
        write_file(&src, "a.txt", b"a");
        let sub = src.join("sub");
        fs::create_dir(&sub).unwrap();
        write_file(&sub, "b.txt", b"b");

        let dst = tmp.path().join("dir_copy");
        copy_item(&src, &dst).unwrap();
        assert_eq!(fs::read(dst.join("a.txt")).unwrap(), b"a");
        assert_eq!(fs::read(dst.join("sub/b.txt")).unwrap(), b"b");
    }

    #[test]
    fn move_item_relocates_and_removes_source() {
        let tmp = TempDir::new().unwrap();
        let src = write_file(tmp.path(), "old.txt", b"data");
        let dst = tmp.path().join("new.txt");
        move_item(&src, &dst).unwrap();
        assert!(!src.exists());
        assert_eq!(fs::read(&dst).unwrap(), b"data");
    }

    #[test]
    fn delete_item_removes_files_and_dirs() {
        let tmp = TempDir::new().unwrap();
        let f = write_file(tmp.path(), "f.txt", b"x");
        delete_item(&f).unwrap();
        assert!(!f.exists());

        let d = tmp.path().join("d");
        fs::create_dir(&d).unwrap();
        write_file(&d, "inner.txt", b"y");
        delete_item(&d).unwrap();
        assert!(!d.exists());
    }

    #[test]
    fn rename_item_renames_within_directory() {
        let tmp = TempDir::new().unwrap();
        write_file(tmp.path(), "before.txt", b"content");
        rename_item(tmp.path(), "before.txt", "after.txt").unwrap();
        assert!(!tmp.path().join("before.txt").exists());
        assert_eq!(fs::read(tmp.path().join("after.txt")).unwrap(), b"content");
    }

    // --- create_dir / touch_file ---

    #[test]
    fn create_dir_makes_nested_directories() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("a/b/c");
        create_dir(&p).unwrap();
        assert!(p.is_dir());
    }

    #[test]
    fn touch_file_creates_and_truncates() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("new.txt");
        touch_file(&p).unwrap();
        assert!(p.exists());
        assert_eq!(fs::metadata(&p).unwrap().len(), 0);

        fs::write(&p, b"existing").unwrap();
        touch_file(&p).unwrap();
        assert_eq!(fs::metadata(&p).unwrap().len(), 0, "touch should truncate");
    }

    // --- batch rename ---

    #[test]
    fn batch_rename_renames_and_counts_failures() {
        let tmp = TempDir::new().unwrap();
        write_file(tmp.path(), "a.txt", b"1");
        write_file(tmp.path(), "b.txt", b"2");
        let items = vec![
            ("a.txt".to_string(), "a_renamed.txt".to_string()),
            ("b.txt".to_string(), "b_renamed.txt".to_string()),
            ("missing.txt".to_string(), "whatever.txt".to_string()),
        ];
        let (success, failed) = batch_rename(tmp.path(), &items).unwrap();
        assert_eq!(success, 2);
        assert_eq!(failed, 1);
        assert!(tmp.path().join("a_renamed.txt").exists());
        assert!(tmp.path().join("b_renamed.txt").exists());
    }

    #[test]
    fn sequential_names_with_brace_placeholder() {
        let names = generate_sequential_names("img_{}.png", 3);
        assert_eq!(names, vec!["img_1.png", "img_2.png", "img_3.png"]);
    }

    #[test]
    fn sequential_names_with_zero_padding() {
        let names = generate_sequential_names("vacation_{:03}.jpg", 3);
        assert_eq!(
            names,
            vec!["vacation_001.jpg", "vacation_002.jpg", "vacation_003.jpg"]
        );
    }

    #[test]
    fn sequential_names_without_placeholder_appends_number() {
        let names = generate_sequential_names("backup", 2);
        assert_eq!(names, vec!["backup_1", "backup_2"]);
    }

    #[test]
    fn batch_rename_with_generated_sequential_pattern() {
        // End-to-end: generate names from a pattern and apply them.
        let tmp = TempDir::new().unwrap();
        write_file(tmp.path(), "x1.txt", b"1");
        write_file(tmp.path(), "x2.txt", b"2");
        write_file(tmp.path(), "x3.txt", b"3");
        let new_names = generate_sequential_names("track_{:02}.txt", 3);
        let items: Vec<(String, String)> = ["x1.txt", "x2.txt", "x3.txt"]
            .iter()
            .map(|s| s.to_string())
            .zip(new_names)
            .collect();
        let (success, failed) = batch_rename(tmp.path(), &items).unwrap();
        assert_eq!((success, failed), (3, 0));
        for expected in ["track_01.txt", "track_02.txt", "track_03.txt"] {
            assert!(tmp.path().join(expected).exists(), "missing {}", expected);
        }
    }

    // --- valid_name / format_err ---

    #[test]
    fn valid_name_rejects_bad_names() {
        assert!(!valid_name(""));
        assert!(!valid_name("a/b"));
        assert!(!valid_name("a\0b"));
        assert!(valid_name("normal file.txt"));
    }

    #[test]
    fn format_err_prefixes_error() {
        let msg = format_err(io::Error::new(io::ErrorKind::NotFound, "gone"));
        assert!(msg.starts_with("Error: "));
        assert!(msg.contains("gone"));
    }
}
