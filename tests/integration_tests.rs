use std::io::{Write, Read};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn test_binary_builds() {
    let output = Command::new("cargo")
        .args(["build"])
        .output()
        .expect("Failed to build");
    assert!(output.status.success(), "Build failed");
}

#[test]
fn test_help_flag() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to run --help");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains("vhs-86") || combined.contains("VHS-86"),
        "Help output should mention VHS-86. Got: {}", combined
    );
}

#[test]
fn test_version_flag() {
    let output = Command::new("cargo")
        .args(["run", "--", "--version"])
        .output()
        .expect("Failed to run --version");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains("0.7.0"),
        "Version output should contain version number. Got: {}", combined
    );
}

#[test]
fn test_config_file_creation_and_loading() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let config_content = r#"
theme = "synthwave"
show_hidden = true

[preview]
syntax_highlight = true
image_preview = false
max_lines = 50

[shell]
cd_on_quit = true
shell_command = "/bin/bash"

[plugins]
enabled = false
auto_load = false
"#;

    std::fs::write(&config_path, config_content).unwrap();
    assert!(config_path.exists());

    let read_content = std::fs::read_to_string(&config_path).unwrap();
    assert!(read_content.contains("synthwave"));
    assert!(read_content.contains("show_hidden"));

    let config = vhs_86::config::Config::load_from(&config_path);
    assert_eq!(config.theme, "synthwave");
    assert!(config.show_hidden);
    assert_eq!(config.preview.max_lines, 50);
}

#[test]
fn test_file_operations_integration() {
    let temp_dir = tempfile::tempdir().unwrap();

    let file1 = temp_dir.path().join("test1.txt");
    let file2 = temp_dir.path().join("test2.txt");
    std::fs::write(&file1, "Hello").unwrap();
    std::fs::write(&file2, "World").unwrap();

    assert!(file1.exists());
    assert!(file2.exists());

    assert_eq!(std::fs::read_to_string(&file1).unwrap(), "Hello");
    assert_eq!(std::fs::read_to_string(&file2).unwrap(), "World");

    let entries: Vec<_> = std::fs::read_dir(temp_dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(entries.len(), 2);
}

#[test]
fn test_archive_operations_integration() {
    let temp_dir = tempfile::tempdir().unwrap();
    let zip_path = temp_dir.path().join("test.zip");

    let file = std::fs::File::create(&zip_path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);
    zip.start_file("hello.txt", options).unwrap();
    zip.write_all(b"Hello from zip!").unwrap();
    zip.finish().unwrap();

    assert!(zip_path.exists());

    let file = std::fs::File::open(&zip_path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    assert_eq!(archive.len(), 1);

    let mut content = String::new();
    archive.by_index(0).unwrap().read_to_string(&mut content).unwrap();
    assert_eq!(content, "Hello from zip!");

    let entries = vhs_86::archive::list_zip_entries(&zip_path).unwrap();
    assert!(!entries.is_empty());
    assert!(entries.iter().any(|e| e.name == "hello.txt"));
}

#[test]
fn test_tar_archive_operations() {
    let temp_dir = tempfile::tempdir().unwrap();
    let tar_path = temp_dir.path().join("test.tar");

    let file = std::fs::File::create(&tar_path).unwrap();
    let mut tar = tar::Builder::new(file);
    let mut header = tar::Header::new_gnu();
    header.set_path("hello.txt").unwrap();
    header.set_size(13);
    header.set_cksum();
    tar.append(&header, b"Hello, World!".as_slice()).unwrap();
    tar.finish().unwrap();

    let entries = vhs_86::archive::list_tar_entries(&tar_path, false).unwrap();
    assert!(!entries.is_empty());
}

#[test]
fn test_permissions_integration() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, "test").unwrap();

    let mut perms = std::fs::metadata(&file_path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&file_path, perms).unwrap();

    let mode = std::fs::metadata(&file_path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o755);
}

#[test]
fn test_directory_navigation() {
    let temp_dir = tempfile::tempdir().unwrap();
    let subdir = temp_dir.path().join("subdir");
    std::fs::create_dir(&subdir).unwrap();

    let nested = subdir.join("nested");
    std::fs::create_dir(&nested).unwrap();

    assert_eq!(nested.parent(), Some(subdir.as_path()));
    assert_eq!(subdir.parent(), Some(temp_dir.path()));

    assert!(nested.exists());
    assert!(subdir.exists());
}

#[test]
fn test_search_functionality() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("searchable.txt");
    std::fs::write(&file_path, "line one\nline two\nline three\n").unwrap();

    let content = std::fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("line two"));
    assert!(!content.contains("line four"));
}

#[test]
fn test_batch_delete_integration() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file1 = temp_dir.path().join("a.txt");
    let file2 = temp_dir.path().join("b.txt");
    std::fs::write(&file1, "a").unwrap();
    std::fs::write(&file2, "b").unwrap();

    let entries: Vec<vhs_86::DirEntry> = vec![
        vhs_86::DirEntry {
            name: "a.txt".to_string(),
            path: file1.clone(),
            kind: vhs_86::EntryKind::File,
            size: 1,
            modified: None,
        },
        vhs_86::DirEntry {
            name: "b.txt".to_string(),
            path: file2.clone(),
            kind: vhs_86::EntryKind::File,
            size: 1,
            modified: None,
        },
    ];

    let refs: Vec<&vhs_86::DirEntry> = entries.iter().collect();
    let action = vhs_86::batch::BatchAction::Delete;
    let (success, failed) = vhs_86::batch::execute_batch_action(&action, &refs).unwrap();
    assert_eq!(success, 2);
    assert_eq!(failed, 0);
    assert!(!file1.exists());
    assert!(!file2.exists());
}

#[test]
fn test_disk_usage_integration() {
    let temp_dir = tempfile::tempdir().unwrap();
    std::fs::write(temp_dir.path().join("large.txt"), "x".repeat(1000)).unwrap();
    std::fs::write(temp_dir.path().join("small.txt"), "x".repeat(100)).unwrap();

    let sizes = vhs_86::disk_usage::analyze_directory(temp_dir.path());
    assert_eq!(sizes.len(), 2);
    assert!(sizes[0].1 >= sizes[1].1);
}

#[test]
fn test_theme_serialization_integration() {
    let theme = vhs_86::theme::Theme::synthwave();
    let serialized = toml::to_string(&theme).unwrap();
    let deserialized: vhs_86::theme::Theme = toml::from_str(&serialized).unwrap();
    assert_eq!(theme.name, deserialized.name);
}

#[test]
fn test_format_size_integration() {
    assert_eq!(vhs_86::format_size(0), "-");
    assert_eq!(vhs_86::format_size(1), "1 B");
    assert_eq!(vhs_86::format_size(1024), "1.0 K");
    assert_eq!(vhs_86::format_size(1024 * 1024), "1.0 M");
}

#[test]
fn test_keybindings_integration() {
    let bindings = vhs_86::keybindings::Keybindings::default();
    use crossterm::event::KeyCode;

    assert_eq!(bindings.get_action(&KeyCode::Char('q')), Some(vhs_86::keybindings::Action::Quit));
    assert_eq!(bindings.get_action(&KeyCode::Char('j')), Some(vhs_86::keybindings::Action::MoveDown));
    assert_eq!(bindings.get_action(&KeyCode::Char('z')), None);
}

#[test]
fn test_permissions_parsing_integration() {
    assert_eq!(vhs_86::permissions::parse_numeric_mode("755"), Some(0o755));
    assert_eq!(vhs_86::permissions::parse_numeric_mode("644"), Some(0o644));
    assert_eq!(vhs_86::permissions::mode_to_string(0o755), "rwxr-xr-x");
}

#[test]
fn test_archive_detection_integration() {
    assert!(vhs_86::archive::detect_archive(PathBuf::from("test.zip").as_path()).is_some());
    assert!(vhs_86::archive::detect_archive(PathBuf::from("test.tar.gz").as_path()).is_some());
    assert!(vhs_86::archive::detect_archive(PathBuf::from("test.txt").as_path()).is_none());
}

#[test]
fn test_preview_integration() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, "Hello, World!\nSecond line\n").unwrap();

    let lines = vhs_86::preview::preview_text_plain(&file_path, 5, 80);
    assert_eq!(lines[0], "Hello, World!");
    assert_eq!(lines[1], "Second line");

    let items = vhs_86::preview::preview_dir(temp_dir.path(), 10);
    assert!(items.iter().any(|i| i.contains("test.txt")));
}

#[test]
fn test_image_detection() {
    assert!(vhs_86::preview::is_image(PathBuf::from("photo.png").as_path()));
    assert!(!vhs_86::preview::is_image(PathBuf::from("file.txt").as_path()));
}

#[test]
fn test_ssh_target_parsing() {
    let (user, host) = vhs_86::remote::parse_ssh_target("alice@example.com").unwrap();
    assert_eq!(user, "alice");
    assert_eq!(host, "example.com");
}
