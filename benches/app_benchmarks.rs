use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::PathBuf;

fn bench_format_size(c: &mut Criterion) {
    c.bench_function("format_size_1kb", |b| {
        b.iter(|| vhs_86::format_size(black_box(1024)))
    });
    c.bench_function("format_size_1mb", |b| {
        b.iter(|| vhs_86::format_size(black_box(1024 * 1024)))
    });
    c.bench_function("format_size_1gb", |b| {
        b.iter(|| vhs_86::format_size(black_box(1024 * 1024 * 1024)))
    });
}

fn bench_format_time(c: &mut Criterion) {
    let now = chrono::Local::now();
    c.bench_function("format_time_some", |b| {
        b.iter(|| vhs_86::format_time(black_box(Some(now))))
    });
    c.bench_function("format_time_none", |b| {
        b.iter(|| vhs_86::format_time(black_box(None)))
    });
}

fn bench_parse_numeric_mode(c: &mut Criterion) {
    c.bench_function("parse_mode_755", |b| {
        b.iter(|| vhs_86::permissions::parse_numeric_mode(black_box("755")))
    });
    c.bench_function("parse_mode_644", |b| {
        b.iter(|| vhs_86::permissions::parse_numeric_mode(black_box("644")))
    });
    c.bench_function("parse_mode_invalid", |b| {
        b.iter(|| vhs_86::permissions::parse_numeric_mode(black_box("abc")))
    });
}

fn bench_mode_to_string(c: &mut Criterion) {
    c.bench_function("mode_to_string_755", |b| {
        b.iter(|| vhs_86::permissions::mode_to_string(black_box(0o755)))
    });
    c.bench_function("mode_to_string_644", |b| {
        b.iter(|| vhs_86::permissions::mode_to_string(black_box(0o644)))
    });
}

fn bench_detect_archive(c: &mut Criterion) {
    c.bench_function("detect_zip", |b| {
        b.iter(|| vhs_86::archive::detect_archive(black_box(std::path::Path::new("test.zip"))))
    });
    c.bench_function("detect_tar", |b| {
        b.iter(|| vhs_86::archive::detect_archive(black_box(std::path::Path::new("test.tar.gz"))))
    });
    c.bench_function("detect_none", |b| {
        b.iter(|| vhs_86::archive::detect_archive(black_box(std::path::Path::new("test.txt"))))
    });
}

fn bench_theme_load(c: &mut Criterion) {
    c.bench_function("theme_load_synthwave", |b| {
        b.iter(|| vhs_86::theme::Theme::load(black_box("synthwave")))
    });
}

fn bench_keycode_to_string(c: &mut Criterion) {
    c.bench_function("keycode_char", |b| {
        b.iter(|| vhs_86::keybindings::keycode_to_string(black_box(&crossterm::event::KeyCode::Char('a'))))
    });
    c.bench_function("keycode_enter", |b| {
        b.iter(|| vhs_86::keybindings::keycode_to_string(black_box(&crossterm::event::KeyCode::Enter)))
    });
}

fn bench_preview_dir(c: &mut Criterion) {
    let temp_dir = tempfile::tempdir().unwrap();
    for i in 0..50 {
        std::fs::write(temp_dir.path().join(format!("file{}.txt", i)), "content").unwrap();
    }
    let path = temp_dir.path().to_path_buf();
    
    c.bench_function("preview_dir_50_files", |b| {
        b.iter(|| vhs_86::preview::preview_dir(black_box(&path), black_box(20)))
    });
}

criterion_group!(
    benches,
    bench_format_size,
    bench_format_time,
    bench_parse_numeric_mode,
    bench_mode_to_string,
    bench_detect_archive,
    bench_theme_load,
    bench_keycode_to_string,
    bench_preview_dir
);
criterion_main!(benches);
