use std::io::{self, Write};
use std::path::Path;

use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use unicode_width::UnicodeWidthStr;

pub fn preview_text_highlighted(
    path: &Path,
    max_lines: usize,
    max_width: usize,
) -> Vec<(String, Option<(u8, u8, u8)>)> {
    let mut lines = Vec::new();

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => {
            lines.push(("[binary or unreadable file]".to_string(), None));
            return lines;
        }
    };

    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let syntax = ss.find_syntax_by_extension(ext).unwrap_or_else(|| ss.find_syntax_plain_text());
    let theme = &ts.themes["base16-ocean.dark"];
    let mut highlighter = HighlightLines::new(syntax, theme);

    for line in LinesWithEndings::from(&content).take(max_lines) {
        let highlighted = highlighter.highlight_line(line, &ss).unwrap_or_default();
        let mut result = String::new();
        let mut last_fg = None;

        for (style, text) in highlighted {
            result.push_str(text);
            if last_fg.is_none() {
                last_fg = Some(style.foreground);
            }
        }

        let mut l = result.trim_end().to_string();
        if l.width() > max_width {
            l = l.chars().take(max_width.saturating_sub(3)).collect::<String>() + "...";
        }

        let color = last_fg.map(|c| (c.r, c.g, c.b));
        lines.push((l, color));
    }

    while lines.len() < max_lines {
        lines.push((String::new(), None));
    }

    lines
}

pub fn preview_text_plain(path: &Path, max_lines: usize, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    if let Ok(content) = std::fs::read_to_string(path) {
        for line in content.lines().take(max_lines) {
            let mut l = line.to_string();
            if l.width() > max_width {
                l = l.chars().take(max_width.saturating_sub(3)).collect::<String>() + "...";
            }
            lines.push(l);
        }
    } else {
        lines.push("[binary or unreadable file]".to_string());
    }
    while lines.len() < max_lines {
        lines.push(String::new());
    }
    lines
}

pub fn is_image(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("bmp") | Some("webp")
    )
}

#[allow(dead_code)]
pub fn send_kitty_image(path: &Path, area: &ratatui::layout::Rect) -> io::Result<()> {
    let img = match image::open(path) {
        Ok(i) => i,
        Err(_) => return Ok(()),
    };

    let (term_w, term_h) = (area.width as u32, area.height as u32);
    if term_w == 0 || term_h == 0 {
        return Ok(());
    }

    // Scale image to fit terminal area
    let img = img.thumbnail(term_w * 8, term_h * 16);
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let raw = rgba.into_raw();

    // Kitty graphics protocol: transmit PNG or raw RGBA
    // Using direct transmission with chunked data
    let data = base64_encode(&raw);
    let mut stdout = io::stdout();

    // Start transmission
    write!(
        stdout,
        "\x1b_Ga=T,f=32,s={},v={},m=1;{}\x1b\\",
        w,
        h,
        &data[..data.len().min(4096)]
    )?;

    // Chunked continuation if needed
    let mut offset = 4096;
    while offset < data.len() {
        let end = (offset + 4096).min(data.len());
        let more = if end < data.len() { 1 } else { 0 };
        write!(stdout, "\x1b_Gm={};{}\x1b\\", more, &data[offset..end])?;
        offset = end;
    }

    stdout.flush()?;
    Ok(())
}

#[allow(dead_code)]
fn base64_encode(data: &[u8]) -> String {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    STANDARD.encode(data)
}

#[allow(dead_code)]
pub fn clear_kitty_image() -> io::Result<()> {
    let mut stdout = io::stdout();
    write!(stdout, "\x1b_Ga=d,d=A\x1b\\")?;
    stdout.flush()?;
    Ok(())
}

pub fn preview_dir(path: &Path, max_items: usize) -> Vec<String> {
    let mut items = Vec::new();
    if let Ok(rd) = std::fs::read_dir(path) {
        for entry in rd.take(max_items) {
            if let Ok(e) = entry {
                let name = e.file_name().to_string_lossy().to_string();
                let icon = if e.metadata().map(|m| m.is_dir()).unwrap_or(false) {
                    "📁"
                } else {
                    "📄"
                };
                items.push(format!("{} {}", icon, name));
            }
        }
    }
    if items.is_empty() {
        items.push("[empty directory]".to_string());
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preview_text_plain() {
        let tmpdir = tempfile::tempdir().unwrap();
        let file_path = tmpdir.path().join("test.txt");
        std::fs::write(&file_path, "line1\nline2\nline3\n").unwrap();

        let lines = preview_text_plain(&file_path, 2, 80);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "line1");
        assert_eq!(lines[1], "line2");
    }

    #[test]
    fn test_preview_text_plain_binary() {
        let tmpdir = tempfile::tempdir().unwrap();
        let file_path = tmpdir.path().join("binary.dat");
        std::fs::write(&file_path, vec![0u8, 1, 2, 255, 254]).unwrap();

        let lines = preview_text_plain(&file_path, 2, 80);
        assert_eq!(lines[0], "[binary or unreadable file]");
    }

    #[test]
    fn test_preview_text_plain_truncation() {
        let tmpdir = tempfile::tempdir().unwrap();
        let file_path = tmpdir.path().join("long.txt");
        std::fs::write(&file_path, "a".repeat(100)).unwrap();

        let lines = preview_text_plain(&file_path, 1, 10);
        assert_eq!(lines[0].len(), 10);
        assert!(lines[0].ends_with("..."));
    }

    #[test]
    fn test_is_image() {
        assert!(is_image(Path::new("photo.png")));
        assert!(is_image(Path::new("photo.jpg")));
        assert!(is_image(Path::new("photo.jpeg")));
        assert!(is_image(Path::new("photo.gif")));
        assert!(is_image(Path::new("photo.bmp")));
        assert!(is_image(Path::new("photo.webp")));
        assert!(!is_image(Path::new("file.txt")));
        assert!(!is_image(Path::new("file.pdf")));
        assert!(!is_image(Path::new("no_extension")));
    }

    #[test]
    fn test_preview_dir() {
        let tmpdir = tempfile::tempdir().unwrap();
        std::fs::write(tmpdir.path().join("a.txt"), "a").unwrap();
        std::fs::create_dir(tmpdir.path().join("subdir")).unwrap();

        let items = preview_dir(tmpdir.path(), 10);
        assert!(!items.is_empty());
        assert!(items.iter().any(|i| i.contains("a.txt")));
        assert!(items.iter().any(|i| i.contains("subdir")));
    }

    #[test]
    fn test_preview_dir_empty() {
        let tmpdir = tempfile::tempdir().unwrap();
        let items = preview_dir(tmpdir.path(), 10);
        assert_eq!(items, vec!["[empty directory]"]);
    }

    #[test]
    fn test_preview_dir_max_items() {
        let tmpdir = tempfile::tempdir().unwrap();
        for i in 0..10 {
            std::fs::write(tmpdir.path().join(format!("{}.txt", i)), "x").unwrap();
        }
        let items = preview_dir(tmpdir.path(), 3);
        assert_eq!(items.len(), 3);
    }
}
