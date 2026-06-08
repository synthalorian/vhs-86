use std::{
    env,
    fs::{self, Metadata},
    io::{self, stdout},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use chrono::{DateTime, Local};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Row, Table, Wrap},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;
use walkdir::WalkDir;

// ═══════════════════════════════════════════════════════════════
//  SYNTHWAVE PALETTE
// ═══════════════════════════════════════════════════════════════
mod palette {
    use ratatui::style::Color;
    pub const BG: Color = Color::Rgb(10, 2, 26);          // deep violet black
    pub const PANEL_BG: Color = Color::Rgb(18, 4, 42);    // panel background
    pub const MAGENTA: Color = Color::Rgb(255, 0, 255);   // hot magenta
    pub const CYAN: Color = Color::Rgb(0, 255, 255);      // electric cyan
    pub const PINK: Color = Color::Rgb(255, 20, 147);     // deep pink
    pub const YELLOW: Color = Color::Rgb(255, 215, 0);    // gold
    pub const ORANGE: Color = Color::Rgb(255, 140, 0);    // dark orange
    pub const WHITE: Color = Color::Rgb(240, 240, 255);   // soft white
    pub const GRAY: Color = Color::Rgb(120, 120, 140);    // muted gray
    pub const GREEN: Color = Color::Rgb(57, 255, 20);     // neon green
    pub const RED: Color = Color::Rgb(255, 50, 80);       // neon red
    pub const BORDER: Color = Color::Rgb(180, 0, 180);    // magenta border
    pub const HIGHLIGHT: Color = Color::Rgb(40, 0, 80);   // selection bg
}

// ═══════════════════════════════════════════════════════════════
//  APP STATE
// ═══════════════════════════════════════════════════════════════
#[derive(Debug, Clone, PartialEq)]
enum EntryKind {
    Dir,
    File,
    Symlink,
    Unknown,
}

#[derive(Debug, Clone)]
struct DirEntry {
    name: String,
    path: PathBuf,
    kind: EntryKind,
    size: u64,
    modified: Option<DateTime<Local>>,
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
}

impl App {
    fn new(start_dir: PathBuf) -> io::Result<Self> {
        let mut app = Self {
            cwd: start_dir.clone(),
            entries: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            show_hidden: false,
            last_key_time: Instant::now(),
            key_buffer: String::new(),
            message: None,
            message_time: None,
            quit: false,
        };
        app.refresh()?;
        Ok(app)
    }

    fn refresh(&mut self) -> io::Result<()> {
        self.entries.clear();
        self.entries.push(DirEntry {
            name: "..".to_string(),
            path: self.cwd.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| self.cwd.clone()),
            kind: EntryKind::Dir,
            size: 0,
            modified: None,
        });

        let mut dirs = Vec::new();
        let mut files = Vec::new();

        for entry in fs::read_dir(&self.cwd)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if !self.show_hidden && name.starts_with('.') {
                continue;
            }
            let meta = entry.metadata().ok();
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

            let de = DirEntry { name, path: entry.path(), kind, size, modified };
            match de.kind {
                EntryKind::Dir => dirs.push(de),
                _ => files.push(de),
            }
        }

        dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        self.entries.extend(dirs);
        self.entries.extend(files);

        self.selected = self.selected.min(self.entries.len().saturating_sub(1));
        self.scroll_offset = self.scroll_offset.min(self.selected);
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
        if let Some(entry) = self.selected_entry() {
            if entry.kind == EntryKind::Dir {
                self.cwd = entry.path.clone();
                self.selected = 0;
                self.scroll_offset = 0;
                self.refresh()?;
            }
        }
        Ok(())
    }

    fn go_home(&mut self) -> io::Result<()> {
        if let Ok(home) = std::env::var("HOME") {
            self.cwd = PathBuf::from(home);
            self.selected = 0;
            self.scroll_offset = 0;
            self.refresh()?;
        }
        Ok(())
    }

    fn toggle_hidden(&mut self) -> io::Result<()> {
        self.show_hidden = !self.show_hidden;
        self.refresh()?;
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

fn preview_text(path: &Path, max_lines: usize, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    if let Ok(content) = fs::read_to_string(path) {
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

fn preview_dir(path: &Path, max_items: usize) -> Vec<String> {
    let mut items = Vec::new();
    if let Ok(rd) = fs::read_dir(path) {
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

// ═══════════════════════════════════════════════════════════════
//  UI RENDERING
// ═══════════════════════════════════════════════════════════════
fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(area);

    let body = main_chunks[0];
    let status_bar = main_chunks[1];

    // Split body into file list (left) and preview (right)
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(body);

    draw_file_list(f, app, h_chunks[0]);
    draw_preview(f, app, h_chunks[1]);
    draw_status_bar(f, app, status_bar);
}

fn draw_file_list(f: &mut Frame, app: &mut App, area: Rect) {
    use palette::*;

    let block = Block::default()
        .title(Span::styled(" VHS-86 ", Style::default().fg(MAGENTA).add_modifier(Modifier::BOLD)))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER));
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
        let (icon, color) = match entry.kind {
            EntryKind::Dir => ("▸", CYAN),
            EntryKind::File => ("•", WHITE),
            EntryKind::Symlink => ("~", PINK),
            EntryKind::Unknown => ("?", GRAY),
        };

        let name = if entry.name.len() > 24 {
            format!("{}...", &entry.name[..21])
        } else {
            entry.name.clone()
        };

        let size = format_size(entry.size);
        let time = format_time(entry.modified);

        let style = if is_selected {
            Style::default().bg(HIGHLIGHT).fg(YELLOW).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(color)
        };

        rows.push(Row::new(vec![
            format!("{} {}", icon, name),
            size,
            time,
        ]).style(style));
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
        Row::new(vec!["Name", "Size", "Modified"])
            .style(Style::default().fg(MAGENTA).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
    )
    .column_spacing(1);

    f.render_widget(table, inner);
}

fn draw_preview(f: &mut Frame, app: &App, area: Rect) {
    use palette::*;

    let block = Block::default()
        .title(Span::styled(" PREVIEW ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let max_lines = inner.height.saturating_sub(2) as usize;
    let max_width = inner.width.saturating_sub(4) as usize;

    let text = if let Some(entry) = app.selected_entry() {
        let mut lines = Vec::new();
        lines.push(Line::from(vec![
            Span::styled("Name: ", Style::default().fg(MAGENTA)),
            Span::styled(&entry.name, Style::default().fg(WHITE).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Path: ", Style::default().fg(MAGENTA)),
            Span::styled(entry.path.to_string_lossy().to_string(), Style::default().fg(GRAY)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Type: ", Style::default().fg(MAGENTA)),
            Span::styled(format!("{:?}", entry.kind), Style::default().fg(YELLOW)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Size: ", Style::default().fg(MAGENTA)),
            Span::styled(format_size(entry.size), Style::default().fg(GREEN)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Modified: ", Style::default().fg(MAGENTA)),
            Span::styled(format_time(entry.modified), Style::default().fg(WHITE)),
        ]));
        lines.push(Line::from(""));

        match entry.kind {
            EntryKind::Dir => {
                lines.push(Line::from(Span::styled("Contents:", Style::default().fg(CYAN).add_modifier(Modifier::BOLD))));
                for item in preview_dir(&entry.path, max_lines.saturating_sub(lines.len())) {
                    lines.push(Line::from(Span::styled(item, Style::default().fg(WHITE))));
                }
            }
            EntryKind::File => {
                lines.push(Line::from(Span::styled("Preview:", Style::default().fg(CYAN).add_modifier(Modifier::BOLD))));
                for pline in preview_text(&entry.path, max_lines.saturating_sub(lines.len()), max_width) {
                    lines.push(Line::from(Span::styled(pline, Style::default().fg(WHITE))));
                }
            }
            _ => {
                lines.push(Line::from(Span::styled("[No preview available]", Style::default().fg(GRAY))));
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
    use palette::*;

    let cwd = app.cwd.to_string_lossy().to_string();
    let left = format!(" {}  |  {} items  |  hidden: {}", cwd, app.entries.len(), app.show_hidden);
    let right = if let Some(ref msg) = app.message {
        format!(" {} ", msg)
    } else {
        " h/j/k/l or ↑↓  |  Enter=open  |  .=toggle hidden  |  ~=home  |  q=quit ".to_string()
    };

    let msg_style = if app.message.is_some() {
        Style::default().bg(RED).fg(WHITE).add_modifier(Modifier::BOLD)
    } else {
        Style::default().bg(PANEL_BG).fg(GRAY)
    };

    let left_width = left.width() as u16;
    let right_width = right.width() as u16;

    let left_span = Span::styled(left, Style::default().bg(PANEL_BG).fg(CYAN));
    let right_span = Span::styled(right, msg_style);
    let mid = area.width.saturating_sub(left_width + right_width);

    let line = Line::from(vec![
        left_span,
        Span::styled(" ".repeat(mid as usize), Style::default().bg(PANEL_BG)),
        right_span,
    ]);

    f.render_widget(Paragraph::new(line), area);
}

// ═══════════════════════════════════════════════════════════════
//  EVENT HANDLING
// ═══════════════════════════════════════════════════════════════
fn handle_events(app: &mut App) -> io::Result<bool> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                return Ok(false);
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        app.quit = true;
                    } else {
                        app.quit = true;
                    }
                }
                KeyCode::Char('j') | KeyCode::Down => app.move_down(1),
                KeyCode::Char('k') | KeyCode::Up => app.move_up(1),
                KeyCode::Char('h') | KeyCode::Left => {
                    let parent = app.cwd.parent().map(|p| p.to_path_buf());
                    if let Some(p) = parent {
                        app.cwd = p;
                        app.selected = 0;
                        app.scroll_offset = 0;
                        app.refresh()?;
                    }
                }
                KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                    app.enter_selected()?;
                }
                KeyCode::Char('g') => {
                    app.selected = 0;
                    app.scroll_offset = 0;
                }
                KeyCode::Char('G') => {
                    if !app.entries.is_empty() {
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
    Ok(app.quit)
}

// ═══════════════════════════════════════════════════════════════
//  MAIN
// ═══════════════════════════════════════════════════════════════
fn main() -> io::Result<()> {
    let start_dir = env::args().nth(1)
        .map(PathBuf::from)
        .or_else(|| env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("/"));

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(start_dir)?;
    let mut last_tick = Instant::now();
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
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        app.quit = true;
                    }
                    KeyCode::Char('j') | KeyCode::Down => app.move_down(1),
                    KeyCode::Char('k') | KeyCode::Up => app.move_up(1),
                    KeyCode::Char('h') | KeyCode::Left => {
                        let parent = app.cwd.parent().map(|p| p.to_path_buf());
                        if let Some(p) = parent {
                            app.cwd = p;
                            app.selected = 0;
                            app.scroll_offset = 0;
                            app.refresh()?;
                        }
                    }
                    KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                        app.enter_selected()?;
                    }
                    KeyCode::Char('g') => {
                        app.selected = 0;
                        app.scroll_offset = 0;
                    }
                    KeyCode::Char('G') => {
                        if !app.entries.is_empty() {
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
