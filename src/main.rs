use std::{
    env,
    fs,
    io::{self, stdout},
    path::PathBuf,
    time::{Duration, Instant},
};

use chrono::{DateTime, Local};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Row, Table, Wrap},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

mod archive;
mod batch;
mod config;
mod disk_usage;
mod git;
mod permissions;
mod preview;
mod remote;
mod theme;

use archive::{detect_archive, list_tar_entries, list_zip_entries};
use batch::{BatchAction, BatchActionType, BatchDialog, BatchSelection};
use config::Config;
use disk_usage::DiskUsageView;
use git::{GitCache, GitStatus};
use permissions::ChmodDialog;
use remote::{parse_ssh_target, RemoteFs};
use theme::Theme;

// ═══════════════════════════════════════════════════════════════
//  APP STATE
// ═══════════════════════════════════════════════════════════════
#[derive(Debug, Clone, PartialEq)]
enum EntryKind {
    Dir,
    File,
    Symlink,
    Unknown,
    Archive,
}

#[derive(Debug, Clone)]
struct DirEntry {
    name: String,
    path: PathBuf,
    kind: EntryKind,
    size: u64,
    modified: Option<DateTime<Local>>,
}

#[derive(Debug, Clone, PartialEq)]
enum InputMode {
    Normal,
    Chmod,
    Batch,
    RemoteConnect,
}

#[derive(Debug, Clone)]
struct ArchiveState {
    archive_path: PathBuf,
    archive_type: archive::ArchiveType,
}

struct App {
    cwd: PathBuf,
    entries: Vec<DirEntry>,
    selected: usize,
    scroll_offset: usize,
    show_hidden: bool,
    last_key_time: Instant,
    key_buffer: String,
    message: Option<String>,
    message_time: Option<Instant>,
    quit: bool,
    config: Config,
    theme: Theme,
    git_cache: GitCache,
    // v0.5.0 features
    archive_state: Option<ArchiveState>,
    remote_fs: RemoteFs,
    chmod_dialog: ChmodDialog,
    disk_usage: DiskUsageView,
    batch_selection: BatchSelection,
    batch_dialog: BatchDialog,
    input_mode: InputMode,
    input_buffer: String,
}

impl App {
    fn new(start_dir: PathBuf, config: Config) -> io::Result<Self> {
        let theme = Theme::load(&config.theme);
        let mut app = Self {
            cwd: start_dir.clone(),
            entries: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            show_hidden: config.show_hidden,
            last_key_time: Instant::now(),
            key_buffer: String::new(),
            message: None,
            message_time: None,
            quit: false,
            config,
            theme,
            git_cache: GitCache::new(),
            archive_state: None,
            remote_fs: RemoteFs::new(),
            chmod_dialog: ChmodDialog::new(),
            disk_usage: DiskUsageView::new(),
            batch_selection: BatchSelection::new(),
            batch_dialog: BatchDialog::new(),
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
        };
        app.refresh()?;
        Ok(app)
    }

    fn refresh(&mut self) -> io::Result<()> {
        self.entries.clear();

        // Handle archive browsing mode
        if let Some(ref archive_state) = self.archive_state {
            self.entries.push(DirEntry {
                name: "..".to_string(),
                path: archive_state.archive_path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| self.cwd.clone()),
                kind: EntryKind::Dir,
                size: 0,
                modified: None,
            });

            let archive_entries = match archive_state.archive_type {
                archive::ArchiveType::Zip => list_zip_entries(&archive_state.archive_path)?,
                archive::ArchiveType::Tar => list_tar_entries(&archive_state.archive_path, false)?,
                archive::ArchiveType::TarGz => list_tar_entries(&archive_state.archive_path, true)?,
            };

            for ae in archive_entries {
                self.entries.push(DirEntry {
                    name: ae.name,
                    path: self.cwd.join(&ae.path),
                    kind: if ae.is_dir { EntryKind::Dir } else { EntryKind::File },
                    size: ae.size,
                    modified: None,
                });
            }

            self.selected = self.selected.min(self.entries.len().saturating_sub(1));
            self.scroll_offset = self.scroll_offset.min(self.selected);
            return Ok(());
        }

        // Handle remote filesystem mode
        if self.remote_fs.is_connected() {
            self.entries.push(DirEntry {
                name: "..".to_string(),
                path: self.cwd.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| self.cwd.clone()),
                kind: EntryKind::Dir,
                size: 0,
                modified: None,
            });

            if let Ok(remote_entries) = self.remote_fs.list_dir(&self.cwd) {
                for entry in remote_entries {
                    self.entries.push(entry);
                }
            }

            self.selected = self.selected.min(self.entries.len().saturating_sub(1));
            self.scroll_offset = self.scroll_offset.min(self.selected);
            return Ok(());
        }

        // Normal filesystem mode
        self.entries.push(DirEntry {
            name: "..".to_string(),
            path: self.cwd.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| self.cwd.clone()),
            kind: EntryKind::Dir,
            size: 0,
            modified: None,
        });

        let mut dirs = Vec::new();
        let mut files = Vec::new();
        let mut archives = Vec::new();

        for entry in fs::read_dir(&self.cwd)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if !self.show_hidden && name.starts_with('.') {
                continue;
            }
            let meta = entry.metadata().ok();
            let path = entry.path();

            // Check if it's an archive
            if detect_archive(&path).is_some() {
                archives.push(DirEntry {
                    name,
                    path,
                    kind: EntryKind::Archive,
                    size: meta.as_ref().map(|m| m.len()).unwrap_or(0),
                    modified: meta.as_ref().and_then(|m| m.modified().ok().map(|t| DateTime::<Local>::from(t))),
                });
                continue;
            }

            let kind = meta.as_ref().map(|m| {
                if m.is_dir() {
                    EntryKind::Dir
                } else if m.is_symlink() {
                    EntryKind::Symlink
                } else if m.is_file() {
                    EntryKind::File
                } else {
                    EntryKind::Unknown
                }
            }).unwrap_or(EntryKind::Unknown);

            let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
            let modified = meta.as_ref().and_then(|m| {
                m.modified().ok().map(|t| DateTime::<Local>::from(t))
            });

            let de = DirEntry { name, path, kind, size, modified };
            match de.kind {
                EntryKind::Dir => dirs.push(de),
                _ => files.push(de),
            }
        }

        dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        archives.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        self.entries.extend(dirs);
        self.entries.extend(files);
        self.entries.extend(archives);

        self.selected = self.selected.min(self.entries.len().saturating_sub(1));
        self.scroll_offset = self.scroll_offset.min(self.selected);

        self.git_cache.refresh(&self.cwd);
        Ok(())
    }

    fn selected_entry(&self) -> Option<&DirEntry> {
        self.entries.get(self.selected)
    }

    fn move_down(&mut self, n: usize) {
        if self.entries.is_empty() { return; }
        self.selected = (self.selected + n).min(self.entries.len() - 1);
    }

    fn move_up(&mut self, n: usize) {
        self.selected = self.selected.saturating_sub(n);
    }

    fn enter_selected(&mut self) -> io::Result<()> {
        let entry_clone = self.entries.get(self.selected).cloned();
        if let Some(entry) = entry_clone {
            if entry.kind == EntryKind::Dir {
                self.cwd = entry.path.clone();
                self.selected = 0;
                self.scroll_offset = 0;
                self.archive_state = None;
                self.refresh()?;
            } else if entry.kind == EntryKind::Archive {
                if let Some(archive_type) = detect_archive(&entry.path) {
                    let type_name = archive::archive_type_name(&archive_type);
                    self.archive_state = Some(ArchiveState {
                        archive_path: entry.path.clone(),
                        archive_type,
                    });
                    self.cwd = entry.path.clone();
                    self.selected = 0;
                    self.scroll_offset = 0;
                    self.set_message(format!("Browsing {} archive", type_name));
                    self.refresh()?;
                }
            }
        }
        Ok(())
    }

    fn go_home(&mut self) -> io::Result<()> {
        if let Ok(home) = std::env::var("HOME") {
            self.cwd = PathBuf::from(home);
            self.selected = 0;
            self.scroll_offset = 0;
            self.archive_state = None;
            self.refresh()?;
        }
        Ok(())
    }

    fn toggle_hidden(&mut self) -> io::Result<()> {
        self.show_hidden = !self.show_hidden;
        self.refresh()?;
        Ok(())
    }

    fn go_parent(&mut self) -> io::Result<()> {
        if self.archive_state.is_some() {
            // Exit archive browsing
            self.archive_state = None;
            if let Some(parent) = self.cwd.parent() {
                self.cwd = parent.to_path_buf();
            }
            self.selected = 0;
            self.scroll_offset = 0;
            self.refresh()?;
        } else if let Some(parent) = self.cwd.parent() {
            self.cwd = parent.to_path_buf();
            self.selected = 0;
            self.scroll_offset = 0;
            self.refresh()?;
        }
        Ok(())
    }

    fn set_message(&mut self, msg: String) {
        self.message = Some(msg);
        self.message_time = Some(Instant::now());
    }

    fn clear_expired_message(&mut self) {
        if let Some(t) = self.message_time {
            if t.elapsed() > Duration::from_secs(3) {
                self.message = None;
                self.message_time = None;
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════
//  UTILITIES
// ═══════════════════════════════════════════════════════════════
fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "K", "M", "G", "T"];
    if size == 0 {
        return "-".to_string();
    }
    let mut size = size as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    if unit_idx == 0 {
        format!("{} {}", size as u64, UNITS[unit_idx])
    } else {
        format!("{:.1} {}", size, UNITS[unit_idx])
    }
}

fn format_time(dt: Option<DateTime<Local>>) -> String {
    dt.map(|d| d.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| "-".to_string())
}

// ═══════════════════════════════════════════════════════════════
//  UI RENDERING
// ═══════════════════════════════════════════════════════════════
fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();

    // Check for modal dialogs
    if app.chmod_dialog.visible {
        draw_chmod_dialog(f, app);
        return;
    }

    if app.batch_dialog.visible {
        draw_batch_dialog(f, app);
        return;
    }

    if app.input_mode == InputMode::RemoteConnect {
        draw_remote_connect_dialog(f, app);
        return;
    }

    if app.disk_usage.visible {
        draw_disk_usage(f, app);
        return;
    }

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(area);

    let body = main_chunks[0];
    let status_bar = main_chunks[1];

    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(body);

    draw_file_list(f, app, h_chunks[0]);
    draw_preview(f, app, h_chunks[1]);
    draw_status_bar(f, app, status_bar);
}

fn draw_file_list(f: &mut Frame, app: &mut App, area: Rect) {
    let theme = &app.theme;

    let mut title = " VHS-86 ".to_string();
    if app.archive_state.is_some() {
        title.push_str(" [ARCHIVE]");
    }
    if app.remote_fs.is_connected() {
        title.push_str(" [REMOTE]");
    }
    if app.batch_selection.active {
        title.push_str(&format!(" [{} selected]", app.batch_selection.count()));
    }

    let block = Block::default()
        .title(Span::styled(title, Style::default().fg(theme.magenta).add_modifier(Modifier::BOLD)))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let visible_rows = inner.height as usize;
    if app.selected >= app.scroll_offset + visible_rows {
        app.scroll_offset = app.selected.saturating_sub(visible_rows - 1);
    } else if app.selected < app.scroll_offset {
        app.scroll_offset = app.selected;
    }

    let mut rows = Vec::new();
    for (idx, entry) in app.entries.iter().enumerate() {
        if idx < app.scroll_offset || idx >= app.scroll_offset + visible_rows {
            continue;
        }
        let is_selected = idx == app.selected;
        let is_batch_selected = app.batch_selection.is_selected(idx);
        let (icon, color) = match entry.kind {
            EntryKind::Dir => ("▸", theme.cyan),
            EntryKind::File => ("•", theme.white),
            EntryKind::Symlink => ("~", theme.pink),
            EntryKind::Archive => ("📦", theme.yellow),
            EntryKind::Unknown => ("?", theme.gray),
        };

        let git_status = app.git_cache.get_status(&entry.path);
        let git_prefix = match git_status {
            GitStatus::Added => "+ ",
            GitStatus::Modified => "M ",
            GitStatus::Untracked => "? ",
            GitStatus::Unchanged => "  ",
        };
        let git_color = match git_status {
            GitStatus::Added => theme.git_added,
            GitStatus::Modified => theme.git_modified,
            GitStatus::Untracked => theme.git_untracked,
            GitStatus::Unchanged => theme.gray,
        };

        let name = if entry.name.len() > 24 {
            format!("{}...", &entry.name[..21])
        } else {
            entry.name.clone()
        };

        let size = format_size(entry.size);
        let time = format_time(entry.modified);

        let mut style = if is_selected {
            Style::default().bg(theme.highlight).fg(theme.yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(color)
        };

        // Add batch selection indicator
        if is_batch_selected {
            style = style.add_modifier(Modifier::UNDERLINED);
        }

        let git_style = if is_selected {
            Style::default().bg(theme.highlight).fg(git_color).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(git_color)
        };

        rows.push(Row::new(vec![
            Line::from(vec![
                Span::styled(git_prefix, git_style),
                Span::styled(format!("{} {}", icon, name), style),
            ]),
            Line::from(Span::styled(size, style)),
            Line::from(Span::styled(time, style)),
        ]));
    }

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(50),
            Constraint::Percentage(20),
            Constraint::Percentage(30),
        ],
    )
    .header(
        Row::new(vec![
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.gray)),
                Span::styled("Name", Style::default().fg(theme.magenta).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
            ]),
            Line::from(Span::styled("Size", Style::default().fg(theme.magenta).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))),
            Line::from(Span::styled("Modified", Style::default().fg(theme.magenta).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))),
        ]),
    )
    .column_spacing(1);

    f.render_widget(table, inner);
}

fn draw_preview(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    let block = Block::default()
        .title(Span::styled(" PREVIEW ", Style::default().fg(theme.cyan).add_modifier(Modifier::BOLD)))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let max_lines = inner.height.saturating_sub(2) as usize;
    let max_width = inner.width.saturating_sub(4) as usize;

    let text = if let Some(entry) = app.selected_entry() {
        let mut lines = Vec::new();
        lines.push(Line::from(vec![
            Span::styled("Name: ", Style::default().fg(theme.magenta)),
            Span::styled(&entry.name, Style::default().fg(theme.white).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Path: ", Style::default().fg(theme.magenta)),
            Span::styled(entry.path.to_string_lossy().to_string(), Style::default().fg(theme.gray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Type: ", Style::default().fg(theme.magenta)),
            Span::styled(format!("{:?}", entry.kind), Style::default().fg(theme.yellow)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Size: ", Style::default().fg(theme.magenta)),
            Span::styled(format_size(entry.size), Style::default().fg(theme.green)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Modified: ", Style::default().fg(theme.magenta)),
            Span::styled(format_time(entry.modified), Style::default().fg(theme.white)),
        ]));

        // Show permissions if available
        if let Some(mode) = permissions::get_file_mode(&entry.path) {
            lines.push(Line::from(vec![
                Span::styled("Permissions: ", Style::default().fg(theme.magenta)),
                Span::styled(
                    format!("{:03o} {}", mode, permissions::mode_to_string(mode)),
                    Style::default().fg(theme.cyan),
                ),
            ]));
        }

        let git_status = app.git_cache.get_status(&entry.path);
        let git_str = match git_status {
            GitStatus::Added => "added",
            GitStatus::Modified => "modified",
            GitStatus::Untracked => "untracked",
            GitStatus::Unchanged => "unchanged",
        };
        let git_color = match git_status {
            GitStatus::Added => theme.git_added,
            GitStatus::Modified => theme.git_modified,
            GitStatus::Untracked => theme.git_untracked,
            GitStatus::Unchanged => theme.gray,
        };
        lines.push(Line::from(vec![
            Span::styled("Git: ", Style::default().fg(theme.magenta)),
            Span::styled(git_str, Style::default().fg(git_color)),
        ]));
        lines.push(Line::from(""));

        match entry.kind {
            EntryKind::Dir => {
                lines.push(Line::from(Span::styled("Contents:", Style::default().fg(theme.cyan).add_modifier(Modifier::BOLD))));
                for item in preview::preview_dir(&entry.path, max_lines.saturating_sub(lines.len())) {
                    lines.push(Line::from(Span::styled(item, Style::default().fg(theme.white))));
                }
            }
            EntryKind::File => {
                if app.config.preview.image_preview && preview::is_image(&entry.path) {
                    lines.push(Line::from(Span::styled("[Image preview - Kitty graphics protocol]", Style::default().fg(theme.cyan))));
                } else if app.config.preview.syntax_highlight {
                    lines.push(Line::from(Span::styled("Preview:", Style::default().fg(theme.cyan).add_modifier(Modifier::BOLD))));
                    for (pline, color) in preview::preview_text_highlighted(&entry.path, max_lines.saturating_sub(lines.len()), max_width) {
                        let style = if let Some((r, g, b)) = color {
                            Style::default().fg(Color::Rgb(r, g, b))
                        } else {
                            Style::default().fg(theme.white)
                        };
                        lines.push(Line::from(Span::styled(pline, style)));
                    }
                } else {
                    lines.push(Line::from(Span::styled("Preview:", Style::default().fg(theme.cyan).add_modifier(Modifier::BOLD))));
                    for pline in preview::preview_text_plain(&entry.path, max_lines.saturating_sub(lines.len()), max_width) {
                        lines.push(Line::from(Span::styled(pline, Style::default().fg(theme.white))));
                    }
                }
            }
            EntryKind::Archive => {
                lines.push(Line::from(Span::styled("Archive Contents:", Style::default().fg(theme.cyan).add_modifier(Modifier::BOLD))));
                if let Some(archive_type) = detect_archive(&entry.path) {
                    let entries_result = match archive_type {
                        archive::ArchiveType::Zip => list_zip_entries(&entry.path),
                        archive::ArchiveType::Tar => list_tar_entries(&entry.path, false),
                        archive::ArchiveType::TarGz => list_tar_entries(&entry.path, true),
                    };
                    match entries_result {
                        Ok(archive_entries) => {
                            for ae in archive_entries.iter().take(max_lines.saturating_sub(lines.len())) {
                                let icon = if ae.is_dir { "▸" } else { "•" };
                                lines.push(Line::from(Span::styled(
                                    format!("{} {} ({})", icon, ae.name, format_size(ae.size)),
                                    Style::default().fg(theme.white),
                                )));
                            }
                        }
                        Err(e) => {
                            lines.push(Line::from(Span::styled(
                                format!("Error reading archive: {}", e),
                                Style::default().fg(theme.red),
                            )));
                        }
                    }
                }
            }
            _ => {
                lines.push(Line::from(Span::styled("[No preview available]", Style::default().fg(theme.gray))));
            }
        }
        Text::from(lines)
    } else {
        Text::from("[No selection]")
    };

    let para = Paragraph::new(text).wrap(Wrap { trim: true });
    f.render_widget(para, inner);
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    let cwd = app.cwd.to_string_lossy().to_string();
    let git_indicator = if app.git_cache.is_repo() { " [git]" } else { "" };
    let batch_indicator = if app.batch_selection.active {
        format!(" [{} selected]", app.batch_selection.count())
    } else {
        String::new()
    };
    let left = format!(" {}  |  {} items  |  hidden: {}{}{}", cwd, app.entries.len(), app.show_hidden, git_indicator, batch_indicator);
    let right = if let Some(ref msg) = app.message {
        format!(" {} ", msg)
    } else {
        " h/j/k/l or ↑↓  |  Enter=open  |  .=toggle hidden  |  ~=home  |  Space=select  |  q=quit ".to_string()
    };

    let msg_style = if app.message.is_some() {
        Style::default().bg(theme.red).fg(theme.white).add_modifier(Modifier::BOLD)
    } else {
        Style::default().bg(theme.panel_bg).fg(theme.gray)
    };

    let left_width = left.width() as u16;
    let right_width = right.width() as u16;

    let left_span = Span::styled(left, Style::default().bg(theme.panel_bg).fg(theme.cyan));
    let right_span = Span::styled(right, msg_style);
    let mid = area.width.saturating_sub(left_width + right_width);

    let line = Line::from(vec![
        left_span,
        Span::styled(" ".repeat(mid as usize), Style::default().bg(theme.panel_bg)),
        right_span,
    ]);

    f.render_widget(Paragraph::new(line), area);
}

fn draw_chmod_dialog(f: &mut Frame, app: &App) {
    let theme = &app.theme;
    let area = f.area();
    let dialog_width = 40u16;
    let dialog_height = 10u16;
    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;
    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    let block = Block::default()
        .title(Span::styled(" CHMOD ", Style::default().fg(theme.red).add_modifier(Modifier::BOLD)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.red));

    let inner = block.inner(dialog_area);
    f.render_widget(block, dialog_area);

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        "Enter numeric mode (e.g. 755):",
        Style::default().fg(theme.white),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!(">>> {}", app.chmod_dialog.input),
        Style::default().fg(theme.yellow).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("Current: {}", app.chmod_dialog.formatted_mode()),
        Style::default().fg(theme.gray),
    )));
    lines.push(Line::from(Span::styled(
        "Enter=apply  Esc=cancel",
        Style::default().fg(theme.gray),
    )));

    let para = Paragraph::new(Text::from(lines));
    f.render_widget(para, inner);
}

fn draw_batch_dialog(f: &mut Frame, app: &App) {
    let theme = &app.theme;
    let area = f.area();
    let dialog_width = 50u16;
    let dialog_height = 8u16;
    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;
    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    let block = Block::default()
        .title(Span::styled(
            format!(" {} ", app.batch_dialog.action_name()),
            Style::default().fg(theme.yellow).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.yellow));

    let inner = block.inner(dialog_area);
    f.render_widget(block, dialog_area);

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        "Enter destination path:",
        Style::default().fg(theme.white),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!(">>> {}", app.batch_dialog.input),
        Style::default().fg(theme.yellow).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Enter=confirm  Esc=cancel",
        Style::default().fg(theme.gray),
    )));

    let para = Paragraph::new(Text::from(lines));
    f.render_widget(para, inner);
}

fn draw_remote_connect_dialog(f: &mut Frame, app: &App) {
    let theme = &app.theme;
    let area = f.area();
    let dialog_width = 50u16;
    let dialog_height = 8u16;
    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;
    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    let block = Block::default()
        .title(Span::styled(" SSH CONNECT ", Style::default().fg(theme.cyan).add_modifier(Modifier::BOLD)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.cyan));

    let inner = block.inner(dialog_area);
    f.render_widget(block, dialog_area);

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        "Enter host (user@host or host):",
        Style::default().fg(theme.white),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!(">>> {}", app.input_buffer),
        Style::default().fg(theme.yellow).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Enter=connect  Esc=cancel",
        Style::default().fg(theme.gray),
    )));

    let para = Paragraph::new(Text::from(lines));
    f.render_widget(para, inner);
}

fn draw_disk_usage(f: &mut Frame, app: &App) {
    let theme = &app.theme;
    let area = f.area();

    let block = Block::default()
        .title(Span::styled(" DISK USAGE ", Style::default().fg(theme.green).add_modifier(Modifier::BOLD)))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.green));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let max_lines = inner.height.saturating_sub(2) as usize;
    let max_width = inner.width.saturating_sub(4) as usize;

    let mut lines = Vec::new();
    if let Some(ref path) = app.disk_usage.path {
        lines.push(Line::from(vec![
            Span::styled("Path: ", Style::default().fg(theme.magenta)),
            Span::styled(path.to_string_lossy().to_string(), Style::default().fg(theme.white)),
        ]));
        lines.push(Line::from(""));

        for (idx, (name, size)) in app.disk_usage.entries.iter().enumerate().take(max_lines.saturating_sub(2)) {
            let is_selected = idx == app.disk_usage.selected;
            let pct = if !app.disk_usage.entries.is_empty() {
                let total: u64 = app.disk_usage.entries.iter().map(|(_, s)| s).sum();
                if total > 0 {
                    *size as f64 / total as f64
                } else {
                    0.0
                }
            } else {
                0.0
            };

            let bar_width = (max_width as f64 * 0.4) as usize;
            let filled = (bar_width as f64 * pct) as usize;
            let empty = bar_width.saturating_sub(filled);
            let bar = "█".repeat(filled) + &"░".repeat(empty);

            let style = if is_selected {
                Style::default().bg(theme.highlight).fg(theme.yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.white)
            };

            lines.push(Line::from(vec![
                Span::styled(format!("{:>2}. ", idx + 1), style),
                Span::styled(format!("{:20} ", truncate(name, 20)), style),
                Span::styled(format!("{:>8} ", format_size(*size)), style),
                Span::styled(format!("{:5.1}% ", pct * 100.0), style),
                Span::styled(bar, Style::default().fg(theme.green)),
            ]));
        }
    } else {
        lines.push(Line::from(Span::styled("No path selected", Style::default().fg(theme.gray))));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "j/k=navigate  q=close",
        Style::default().fg(theme.gray),
    )));

    let para = Paragraph::new(Text::from(lines));
    f.render_widget(para, inner);
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    } else {
        s.to_string()
    }
}

// ═══════════════════════════════════════════════════════════════
//  MAIN
// ═══════════════════════════════════════════════════════════════
fn main() -> io::Result<()> {
    let config = Config::load();
    let start_dir = env::args().nth(1)
        .map(PathBuf::from)
        .or_else(|| env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("/"));

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(start_dir, config)?;
    let tick_rate = Duration::from_millis(100);

    let res = run_app(&mut terminal, &mut app, tick_rate);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    // Shell integration: cd on quit
    if app.config.shell.cd_on_quit && !app.remote_fs.is_connected() {
        println!("{}", app.cwd.display());
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| draw(f, app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = crossterm::event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // Handle input modes
                match app.input_mode {
                    InputMode::Chmod => {
                        match key.code {
                            KeyCode::Esc => {
                                app.input_mode = InputMode::Normal;
                                app.chmod_dialog.close();
                            }
                            KeyCode::Enter => {
                                if let Err(e) = app.chmod_dialog.apply() {
                                    app.set_message(format!("Chmod error: {}", e));
                                } else {
                                    app.set_message("Permissions updated".to_string());
                                    app.refresh()?;
                                }
                                app.input_mode = InputMode::Normal;
                                app.chmod_dialog.close();
                            }
                            KeyCode::Backspace => {
                                app.chmod_dialog.pop_char();
                            }
                            KeyCode::Char(c) => {
                                app.chmod_dialog.push_char(c);
                            }
                            _ => {}
                        }
                        continue;
                    }
                    InputMode::Batch => {
                        match key.code {
                            KeyCode::Esc => {
                                app.input_mode = InputMode::Normal;
                                app.batch_dialog.close();
                            }
                            KeyCode::Enter => {
                                let dest = PathBuf::from(&app.batch_dialog.input);
                                let action = match app.batch_dialog.action {
                                    Some(BatchActionType::Delete) => BatchAction::Delete,
                                    Some(BatchActionType::Copy) => BatchAction::Copy { dest },
                                    Some(BatchActionType::Move) => BatchAction::Move { dest },
                                    None => {
                                        app.input_mode = InputMode::Normal;
                                        app.batch_dialog.close();
                                        continue;
                                    }
                                };
                                let selected: Vec<&DirEntry> = app.batch_selection.get_selected_entries(&app.entries);
                                match batch::execute_batch_action(&action, &selected) {
                                    Ok((success, failed)) => {
                                        app.set_message(format!("Batch: {} succeeded, {} failed", success, failed));
                                        app.batch_selection.clear();
                                        app.refresh()?;
                                    }
                                    Err(e) => {
                                        app.set_message(format!("Batch error: {}", e));
                                    }
                                }
                                app.input_mode = InputMode::Normal;
                                app.batch_dialog.close();
                            }
                            KeyCode::Backspace => {
                                app.batch_dialog.pop_char();
                            }
                            KeyCode::Char(c) => {
                                app.batch_dialog.push_char(c);
                            }
                            _ => {}
                        }
                        continue;
                    }
                    InputMode::RemoteConnect => {
                        match key.code {
                            KeyCode::Esc => {
                                app.input_mode = InputMode::Normal;
                                app.input_buffer.clear();
                            }
                            KeyCode::Enter => {
                                if let Some((user, host)) = parse_ssh_target(&app.input_buffer) {
                                    match app.remote_fs.connect(&host, &user) {
                                        Ok(_) => {
                                            app.set_message(format!("Connected to {}@{}", user, host));
                                            app.cwd = PathBuf::from("/home").join(&user);
                                            app.selected = 0;
                                            app.scroll_offset = 0;
                                            app.refresh()?;
                                        }
                                        Err(e) => {
                                            app.set_message(format!("Connection failed: {}", e));
                                        }
                                    }
                                } else {
                                    app.set_message("Invalid SSH target".to_string());
                                }
                                app.input_mode = InputMode::Normal;
                                app.input_buffer.clear();
                            }
                            KeyCode::Backspace => {
                                app.input_buffer.pop();
                            }
                            KeyCode::Char(c) => {
                                app.input_buffer.push(c);
                            }
                            _ => {}
                        }
                        continue;
                    }
                    InputMode::Normal => {}
                }

                // Normal mode key handling
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        if app.disk_usage.visible {
                            app.disk_usage.close();
                        } else {
                            app.quit = true;
                        }
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        if app.disk_usage.visible {
                            app.disk_usage.move_down();
                        } else {
                            app.move_down(1);
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        if app.disk_usage.visible {
                            app.disk_usage.move_up();
                        } else {
                            app.move_up(1);
                        }
                    }
                    KeyCode::Char('h') | KeyCode::Left => {
                        app.go_parent()?;
                    }
                    KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                        if app.disk_usage.visible {
                            // Do nothing in disk usage view
                        } else {
                            app.enter_selected()?;
                        }
                    }
                    KeyCode::Char('g') => {
                        if app.disk_usage.visible {
                            app.disk_usage.selected = 0;
                        } else {
                            app.selected = 0;
                            app.scroll_offset = 0;
                        }
                    }
                    KeyCode::Char('G') => {
                        if app.disk_usage.visible {
                            if !app.disk_usage.entries.is_empty() {
                                app.disk_usage.selected = app.disk_usage.entries.len() - 1;
                            }
                        } else if !app.entries.is_empty() {
                            app.selected = app.entries.len() - 1;
                        }
                    }
                    KeyCode::Char('~') | KeyCode::Char('`') => {
                        app.go_home()?;
                    }
                    KeyCode::Char('.') => {
                        app.toggle_hidden()?;
                        app.set_message(format!("Hidden files: {}", if app.show_hidden { "ON" } else { "OFF" }));
                    }
                    KeyCode::Char(' ') => {
                        // Toggle batch selection
                        app.batch_selection.toggle(app.selected);
                        let count = app.batch_selection.count();
                        app.set_message(format!("{} items selected", count));
                    }
                    KeyCode::Char('c') => {
                        if let Some(entry) = app.entries.get(app.selected) {
                            let path = entry.path.clone();
                            app.input_mode = InputMode::Chmod;
                            app.chmod_dialog.open(&path);
                        }
                    }
                    KeyCode::Char('d') => {
                        if app.disk_usage.visible {
                            app.disk_usage.close();
                        } else {
                            app.disk_usage.open(&app.cwd);
                            app.set_message("Disk usage analyzer".to_string());
                        }
                    }
                    KeyCode::Char('r') => {
                        if app.remote_fs.is_connected() {
                            app.remote_fs.disconnect();
                            app.set_message("Disconnected from remote".to_string());
                            app.go_home()?;
                        } else {
                            app.input_mode = InputMode::RemoteConnect;
                            app.input_buffer.clear();
                        }
                    }
                    KeyCode::Char('D') => {
                        // Batch delete
                        if app.batch_selection.active {
                            app.input_mode = InputMode::Batch;
                            app.batch_dialog.open(BatchActionType::Delete);
                        } else if let Some(entry) = app.selected_entry() {
                            if entry.name != ".." {
                                match fs::remove_file(&entry.path) {
                                    Ok(_) => {
                                        app.set_message(format!("Deleted: {}", entry.name));
                                        app.refresh()?;
                                    }
                                    Err(e) => {
                                        app.set_message(format!("Delete failed: {}", e));
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Char('C') => {
                        // Batch copy
                        if app.batch_selection.active {
                            app.input_mode = InputMode::Batch;
                            app.batch_dialog.open(BatchActionType::Copy);
                        }
                    }
                    KeyCode::Char('M') => {
                        // Batch move
                        if app.batch_selection.active {
                            app.input_mode = InputMode::Batch;
                            app.batch_dialog.open(BatchActionType::Move);
                        }
                    }
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        let now = Instant::now();
                        if now.duration_since(app.last_key_time) > Duration::from_millis(800) {
                            app.key_buffer.clear();
                        }
                        app.last_key_time = now;
                        app.key_buffer.push(c);
                        if let Ok(n) = app.key_buffer.parse::<usize>() {
                            if n > 0 && n <= app.entries.len() {
                                app.selected = n - 1;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.clear_expired_message();
            last_tick = Instant::now();
        }

        if app.quit {
            return Ok(());
        }
    }
}
