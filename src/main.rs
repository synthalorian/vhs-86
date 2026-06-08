mod config;
mod files;

use config::{parse_color, Config, KeyCodeChar};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use notify::{Config as NotifyConfig, Event as NotifyEvent, RecommendedWatcher, RecursiveMode, Watcher};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Terminal,
};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol, StatefulImage};
use std::collections::HashSet;
use std::io::{self, stdout};
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use std::sync::mpsc;
use std::time::{Duration, Instant, SystemTime};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use walkdir::WalkDir;

/// Pending file operation stored in the clipboard.
#[derive(Debug, Clone)]
enum Operation {
    Copy,
    Move,
}

/// UI mode driving input handling and rendering.
#[derive(Debug, Clone)]
enum Mode {
    Normal,
    Search,
    Rename(String),      // old name
    BatchRename,         // batch rename pattern input
    Confirm(ConfirmAction),
    Command(String),     // command input
    ShellInput(String),  // shell command input
    ShellOutput(String), // display shell output
    FileViewer,          // view file content with syntax highlighting or hex
    FuzzyFinder(String), // recursive fuzzy finder
    Bookmarks,           // bookmark list and jump
}

/// Actions that require a y/n confirmation.
#[derive(Debug, Clone)]
enum ConfirmAction {
    Delete(PathBuf),
    DeleteBatch(Vec<PathBuf>),
    Overwrite(PathBuf, PathBuf, Operation),
}

/// File sorting mode.
#[derive(Debug, Clone, Copy, PartialEq)]
enum SortBy {
    Name,
    Size,
    Modified,
}

/// Main application state.
struct App {
    current_dir: PathBuf,
    files: Vec<(String, bool)>,
    selected: usize,
    show_hidden: bool,
    config: Config,
    mode: Mode,
    clipboard: Vec<(PathBuf, Operation)>,
    status_message: Option<(String, Instant)>,
    search_query: String,
    search_matches: Vec<usize>,
    search_selected: usize,
    rename_input: String,
    selected_indices: HashSet<usize>, // batch selection
    themes: Vec<(String, Config)>,    // available themes
    sort_by: SortBy,
    filter_ext: Option<String>,
    list_height: u16,
    g_pending: bool,
    picker: Picker,
    image_state: Option<StatefulProtocol>,
    last_image_path: Option<PathBuf>,
    batch_rename_input: String,
    file_viewer_content: Vec<(String, Style)>,
    file_viewer_scroll: usize,
    file_viewer_title: String,
    fuzzy_finder_paths: Vec<PathBuf>,
    fuzzy_finder_matches: Vec<usize>,
    fuzzy_finder_selected: usize,
    shell_input: String,
    shell_scroll: usize,
    bookmark_selected: usize,
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    config_rx: Option<mpsc::Receiver<()>>,
    config_last_modified: Option<SystemTime>,
}

impl App {
    fn new(initial_dir: PathBuf, config: Config) -> Self {
        let themes = config::load_themes();
        let picker = Picker::from_query_stdio()
            .unwrap_or_else(|_| Picker::halfblocks());
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        let config_rx = setup_config_watcher();
        let mut app = Self {
            current_dir: initial_dir.clone(),
            files: Self::list_files(&initial_dir, true, SortBy::Name, None),
            selected: 0,
            show_hidden: true,
            config,
            mode: Mode::Normal,
            clipboard: Vec::new(),
            status_message: None,
            search_query: String::new(),
            search_matches: Vec::new(),
            search_selected: 0,
            rename_input: String::new(),
            selected_indices: HashSet::new(),
            themes,
            sort_by: SortBy::Name,
            filter_ext: None,
            list_height: 10,
            g_pending: false,
            picker,
            image_state: None,
            last_image_path: None,
            batch_rename_input: String::new(),
            file_viewer_content: Vec::new(),
            file_viewer_scroll: 0,
            file_viewer_title: String::new(),
            fuzzy_finder_paths: Vec::new(),
            fuzzy_finder_matches: Vec::new(),
            fuzzy_finder_selected: 0,
            shell_input: String::new(),
            shell_scroll: 0,
            bookmark_selected: 0,
            syntax_set,
            theme_set,
            config_rx,
            config_last_modified: None,
        };
        app.clamp_selection();
        app
    }

    fn list_files(
        dir: &Path,
        show_hidden: bool,
        sort_by: SortBy,
        filter_ext: Option<&str>,
    ) -> Vec<(String, bool)> {
        let mut files = vec![];
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !show_hidden && name.starts_with('.') && name != "." {
                    continue;
                }
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);

                // Apply extension filter (only to files, not directories)
                if !is_dir {
                    if let Some(ext_filter) = filter_ext {
                        let file_ext = Path::new(&name)
                            .extension()
                            .and_then(|e| e.to_str())
                            .map(|e| e.to_ascii_lowercase());
                        if file_ext != Some(ext_filter.to_ascii_lowercase()) {
                            continue;
                        }
                    }
                }

                files.push((name, is_dir));
            }
        }
        files.sort_by(|a, b| {
            // Directories first
            match (a.1, b.1) {
                (true, false) => return std::cmp::Ordering::Less,
                (false, true) => return std::cmp::Ordering::Greater,
                _ => {}
            }
            // Then by selected sort mode
            match sort_by {
                SortBy::Name => a.0.cmp(&b.0),
                SortBy::Size => {
                    let size_a = std::fs::metadata(dir.join(&a.0)).map(|m| m.len()).unwrap_or(0);
                    let size_b = std::fs::metadata(dir.join(&b.0)).map(|m| m.len()).unwrap_or(0);
                    size_a.cmp(&size_b)
                }
                SortBy::Modified => {
                    let time_a = std::fs::metadata(dir.join(&a.0))
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    let time_b = std::fs::metadata(dir.join(&b.0))
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    time_a.cmp(&time_b)
                }
            }
        });
        files
    }

    fn refresh(&mut self) {
        self.files = Self::list_files(
            &self.current_dir,
            self.show_hidden,
            self.sort_by,
            self.filter_ext.as_deref(),
        );
        self.clamp_selection();
        self.selected_indices.clear();
    }

    fn clamp_selection(&mut self) {
        if self.files.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.files.len() {
            self.selected = self.files.len() - 1;
        }
    }

    fn selected_path(&self) -> Option<PathBuf> {
        self.files
            .get(self.selected)
            .map(|(name, _)| self.current_dir.join(name))
    }

    fn selected_name(&self) -> Option<&String> {
        self.files.get(self.selected).map(|(name, _)| name)
    }

    fn set_status(&mut self, msg: String) {
        self.status_message = Some((msg, Instant::now() + Duration::from_secs(4)));
    }

    fn clear_status_if_expired(&mut self) {
        if let Some((_, until)) = &self.status_message
            && Instant::now() > *until
        {
            self.status_message = None;
        }
    }

    fn update_image_preview(&mut self) {
        let current = self.selected_path();
        if current == self.last_image_path {
            return;
        }
        self.last_image_path = current.clone();
        self.image_state = None;

        if let Some(ref path) = current {
            if path.is_file() && files::is_image_file(path) {
                if let Ok(reader) = image::ImageReader::open(path) {
                    if let Ok(dyn_img) = reader.decode() {
                        self.image_state = Some(self.picker.new_resize_protocol(dyn_img));
                    }
                }
            }
        }
    }

    fn check_config_reload(&mut self) {
        if let Some(ref rx) = self.config_rx {
            if rx.try_recv().is_ok() {
                std::thread::sleep(Duration::from_millis(100));
                let new_config = config::load_config();
                self.config.colors = new_config.colors.clone();
                self.config.keys = new_config.keys.clone();
                self.config.active_theme = new_config.active_theme.clone();
                self.config.bookmarks = new_config.bookmarks.clone();
                self.set_status("Config reloaded".to_string());
            }
        }
    }

    fn open_file_viewer(&mut self) {
        if let Some(path) = self.selected_path() {
            if !path.is_file() {
                self.set_status("Not a file".to_string());
                return;
            }
            self.file_viewer_title = path.display().to_string();
            self.file_viewer_scroll = 0;
            self.file_viewer_content.clear();

            if files::is_text_file(&path) {
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                        let syntax = self.syntax_set.find_syntax_by_extension(ext)
                            .or_else(|| self.syntax_set.find_syntax_by_first_line(&content))
                            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
                        let theme = &self.theme_set.themes["base16-ocean.dark"];
                        let mut highlighter = HighlightLines::new(syntax, theme);
                        for line in LinesWithEndings::from(&content) {
                            let highlighted = highlighter.highlight_line(line, &self.syntax_set).unwrap_or_default();
                            let mut line_text = String::new();
                            let mut line_style = Style::default();
                            if highlighted.is_empty() {
                                line_text = line.trim_end_matches('\n').to_string();
                            } else {
                                for (style, text) in highlighted {
                                    line_text.push_str(text);
                                    line_style = Style::default()
                                        .fg(Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b));
                                }
                            }
                            self.file_viewer_content.push((line_text.trim_end_matches('\n').to_string(), line_style));
                        }
                        self.mode = Mode::FileViewer;
                    }
                    Err(e) => self.set_status(files::format_err(e)),
                }
            } else {
                match files::hex_dump(&path, 2048) {
                    Ok(hex) => {
                        for line in hex.lines() {
                            self.file_viewer_content.push((line.to_string(), Style::default().fg(Color::Yellow)));
                        }
                        self.mode = Mode::FileViewer;
                    }
                    Err(e) => self.set_status(files::format_err(e)),
                }
            }
        }
    }

    fn open_fuzzy_finder(&mut self) {
        self.fuzzy_finder_paths.clear();
        self.fuzzy_finder_matches.clear();
        self.fuzzy_finder_selected = 0;
        for entry in WalkDir::new(&self.current_dir).into_iter().flatten() {
            let path = entry.path().to_path_buf();
            if path != self.current_dir {
                self.fuzzy_finder_paths.push(path);
            }
        }
        self.mode = Mode::FuzzyFinder(String::new());
        self.update_fuzzy_finder();
    }

    fn update_fuzzy_finder(&mut self) {
        let matcher = SkimMatcherV2::default();
        let query = match &self.mode {
            Mode::FuzzyFinder(q) => q.to_lowercase(),
            _ => return,
        };
        self.fuzzy_finder_matches = self.fuzzy_finder_paths
            .iter()
            .enumerate()
            .filter_map(|(i, path)| {
                let name = path.strip_prefix(&self.current_dir)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .to_string();
                matcher.fuzzy_match(&name.to_lowercase(), &query).map(|_| i)
            })
            .collect();
        self.fuzzy_finder_selected = 0;
    }

    fn run_shell_command(&mut self, cmd: &str) {
        if cmd.trim().is_empty() {
            return;
        }
        let output = StdCommand::new("sh")
            .arg("-c")
            .arg(cmd)
            .current_dir(&self.current_dir)
            .output();
        match output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                let stderr = String::from_utf8_lossy(&result.stderr);
                let mut combined = format!("$ {}\n", cmd);
                if !stdout.is_empty() {
                    combined.push_str(&stdout);
                }
                if !stderr.is_empty() {
                    combined.push_str("\n[stderr]\n");
                    combined.push_str(&stderr);
                }
                if stdout.is_empty() && stderr.is_empty() {
                    combined.push_str("(no output)");
                }
                self.shell_scroll = 0;
                self.mode = Mode::ShellOutput(combined);
                self.refresh();
            }
            Err(e) => self.set_status(format!("Shell error: {}", e)),
        }
    }

    fn jump_to_bookmark(&mut self, index: usize) {
        if let Some(path_str) = self.config.bookmarks.get(index) {
            let path = PathBuf::from(path_str);
            if path.is_dir() {
                self.current_dir = path;
                self.refresh();
                self.selected = 0;
                self.mode = Mode::Normal;
            } else {
                self.set_status(format!("Bookmark not found: {}", path_str));
            }
        }
    }

    fn add_bookmark(&mut self) {
        let path_str = self.current_dir.to_string_lossy().to_string();
        if self.config.bookmarks.contains(&path_str) {
            self.set_status("Directory already bookmarked".to_string());
            return;
        }
        self.config.bookmarks.push(path_str.clone());
        config::save_config(&self.config);
        self.set_status(format!("Bookmarked: {}", path_str));
    }

    fn remove_bookmark(&mut self, index: usize) {
        if index < self.config.bookmarks.len() {
            let removed = self.config.bookmarks.remove(index);
            config::save_config(&self.config);
            self.set_status(format!("Removed bookmark: {}", removed));
            if self.bookmark_selected >= self.config.bookmarks.len() && self.bookmark_selected > 0 {
                self.bookmark_selected -= 1;
            }
        }
    }

    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        let key = KeyCodeChar::from(code);

        match &mut self.mode {
            Mode::Confirm(action) => {
                if key == KeyCodeChar::Char("y".to_string())
                    || key == KeyCodeChar::Char("Y".to_string())
                    || key == KeyCodeChar::Enter
                {
                    let action = action.clone();
                    self.execute_confirm(action);
                }
                self.mode = Mode::Normal;
            }
            Mode::Rename(old) => match code {
                KeyCode::Esc => self.mode = Mode::Normal,
                KeyCode::Enter => {
                    let old_name = old.clone();
                    let new_name = self.rename_input.clone();
                    self.rename_input.clear();
                    self.perform_rename(&old_name, &new_name);
                    self.mode = Mode::Normal;
                }
                KeyCode::Backspace => {
                    self.rename_input.pop();
                }
                KeyCode::Char(c) => self.rename_input.push(c),
                _ => {}
            },
            Mode::BatchRename => match code {
                KeyCode::Esc => {
                    self.batch_rename_input.clear();
                    self.mode = Mode::Normal;
                }
                KeyCode::Enter => {
                    let pattern = self.batch_rename_input.clone();
                    self.batch_rename_input.clear();
                    self.perform_batch_rename(&pattern);
                    self.mode = Mode::Normal;
                }
                KeyCode::Backspace => {
                    self.batch_rename_input.pop();
                }
                KeyCode::Char(c) => self.batch_rename_input.push(c),
                _ => {}
            },
            Mode::Search => match code {
                KeyCode::Esc => {
                    self.search_query.clear();
                    self.search_matches.clear();
                    self.mode = Mode::Normal;
                }
                KeyCode::Enter => {
                    if let Some(&idx) = self.search_matches.get(self.search_selected) {
                        self.selected = idx;
                        self.clamp_selection();
                    }
                    self.search_query.clear();
                    self.search_matches.clear();
                    self.mode = Mode::Normal;
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.update_search();
                }
                KeyCode::Down => {
                    if self.search_selected + 1 < self.search_matches.len() {
                        self.search_selected += 1;
                    }
                }
                KeyCode::Up => {
                    if self.search_selected > 0 {
                        self.search_selected -= 1;
                    }
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                    self.update_search();
                }
                _ => {}
            },
            Mode::Command(input) => match code {
                KeyCode::Esc => {
                    self.mode = Mode::Normal;
                }
                KeyCode::Enter => {
                    let cmd = input.clone();
                    self.mode = Mode::Normal;
                    self.execute_command(&cmd);
                }
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Char(c) => {
                    input.push(c);
                }
                _ => {}
            },
            Mode::ShellInput(input) => match code {
                KeyCode::Esc => {
                    self.mode = Mode::Normal;
                }
                KeyCode::Enter => {
                    let cmd = input.clone();
                    self.mode = Mode::Normal;
                    self.run_shell_command(&cmd);
                }
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Char(c) => {
                    input.push(c);
                }
                _ => {}
            },
            Mode::ShellOutput(_) => match code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.mode = Mode::Normal;
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.shell_scroll += 1;
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if self.shell_scroll > 0 {
                        self.shell_scroll -= 1;
                    }
                }
                _ => {}
            },
            Mode::FileViewer => match code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.mode = Mode::Normal;
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.file_viewer_scroll += 1;
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if self.file_viewer_scroll > 0 {
                        self.file_viewer_scroll -= 1;
                    }
                }
                KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) => {
                    self.file_viewer_scroll += self.list_height as usize;
                }
                KeyCode::Char('u') if modifiers.contains(KeyModifiers::CONTROL) => {
                    self.file_viewer_scroll = self.file_viewer_scroll.saturating_sub(self.list_height as usize);
                }
                KeyCode::Char('g') if modifiers == KeyModifiers::NONE => {
                    self.file_viewer_scroll = 0;
                }
                KeyCode::Char('G') => {
                    self.file_viewer_scroll = self.file_viewer_content.len().saturating_sub(1);
                }
                _ => {}
            },
            Mode::FuzzyFinder(query) => match code {
                KeyCode::Esc => {
                    self.mode = Mode::Normal;
                }
                KeyCode::Enter => {
                    let selected_path = self.fuzzy_finder_matches.get(self.fuzzy_finder_selected)
                        .and_then(|&idx| self.fuzzy_finder_paths.get(idx))
                        .cloned();
                    if let Some(path) = selected_path {
                        if path.is_dir() {
                            self.current_dir = path;
                            self.refresh();
                            self.selected = 0;
                        } else if let Some(parent) = path.parent() {
                            let file_name = path.file_name().map(|n| n.to_string_lossy().to_string());
                            self.current_dir = parent.to_path_buf();
                            self.refresh();
                            if let Some(name) = file_name {
                                self.selected = self.files.iter().position(|(n, _)| n == &name).unwrap_or(0);
                            }
                        }
                    }
                    self.mode = Mode::Normal;
                }
                KeyCode::Backspace => {
                    query.pop();
                    self.update_fuzzy_finder();
                }
                KeyCode::Down => {
                    if self.fuzzy_finder_selected + 1 < self.fuzzy_finder_matches.len() {
                        self.fuzzy_finder_selected += 1;
                    }
                }
                KeyCode::Up => {
                    if self.fuzzy_finder_selected > 0 {
                        self.fuzzy_finder_selected -= 1;
                    }
                }
                KeyCode::Char(c) => {
                    query.push(c);
                    self.update_fuzzy_finder();
                }
                _ => {}
            },
            Mode::Bookmarks => match code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.mode = Mode::Normal;
                }
                KeyCode::Enter => {
                    self.jump_to_bookmark(self.bookmark_selected);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.bookmark_selected + 1 < self.config.bookmarks.len() {
                        self.bookmark_selected += 1;
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.bookmark_selected > 0 {
                        self.bookmark_selected -= 1;
                    }
                }
                KeyCode::Char('d') => {
                    self.remove_bookmark(self.bookmark_selected);
                }
                _ => {}
            },
            Mode::Normal => self.handle_normal_key(code, &key, modifiers),
        }
    }

    fn handle_normal_key(&mut self, code: KeyCode, key: &KeyCodeChar, modifiers: KeyModifiers) {
        let cfg = &self.config;

        // Handle g-pending chord (gg)
        if self.g_pending {
            self.g_pending = false;
            if code == KeyCode::Char('g') && modifiers == KeyModifiers::NONE {
                self.selected = 0;
                return;
            }
            // Fall through for other keys
        }

        // Quit
        if matches!(code, KeyCode::Char('q') | KeyCode::Esc)
            || config::key_matches(&cfg.keys.quit, key)
        {
            std::process::exit(0);
        }

        // Navigation
        if matches!(code, KeyCode::Down | KeyCode::Char('j'))
            || config::key_matches(&cfg.keys.down, key)
        {
            if self.selected + 1 < self.files.len() {
                self.selected += 1;
            }
            return;
        }
        if matches!(code, KeyCode::Up | KeyCode::Char('k'))
            || config::key_matches(&cfg.keys.up, key)
        {
            if self.selected > 0 {
                self.selected -= 1;
            }
            return;
        }
        if matches!(code, KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter)
            || config::key_matches(&cfg.keys.right, key)
        {
            if let Some((name, is_dir)) = self.files.get(self.selected) {
                let path = self.current_dir.join(name);
                if *is_dir {
                    self.current_dir = path;
                    self.refresh();
                    self.selected = 0;
                }
            }
            return;
        }
        if matches!(code, KeyCode::Left | KeyCode::Char('h'))
            || config::key_matches(&cfg.keys.left, key)
        {
            if let Some(parent) = self.current_dir.parent() {
                self.current_dir = parent.to_path_buf();
                self.refresh();
                self.selected = 0;
            }
            return;
        }

        // Vim-style gg / G
        if code == KeyCode::Char('g') && modifiers == KeyModifiers::NONE {
            self.g_pending = true;
            return;
        }
        if code == KeyCode::Char('G') || config::key_matches(&cfg.keys.bottom, key) {
            if !self.files.is_empty() {
                self.selected = self.files.len() - 1;
            }
            return;
        }

        // Ctrl+d / Ctrl+u half-page scroll
        if code == KeyCode::Char('d') && modifiers.contains(KeyModifiers::CONTROL) {
            let half_page = (self.list_height / 2).max(1) as usize;
            self.selected = (self.selected + half_page).min(self.files.len().saturating_sub(1));
            return;
        }
        if code == KeyCode::Char('u') && modifiers.contains(KeyModifiers::CONTROL) {
            let half_page = (self.list_height / 2).max(1) as usize;
            self.selected = self.selected.saturating_sub(half_page);
            return;
        }

        if code == KeyCode::Char('~') || config::key_matches(&cfg.keys.home, key) {
            if let Some(home) = dirs::home_dir() {
                self.current_dir = home;
                self.refresh();
                self.selected = 0;
            }
            return;
        }
        if code == KeyCode::Char('.') || config::key_matches(&cfg.keys.toggle_hidden, key) {
            self.show_hidden = !self.show_hidden;
            self.refresh();
            return;
        }

        // Search
        if code == KeyCode::Char('/') || config::key_matches(&cfg.keys.search, key) {
            self.mode = Mode::Search;
            self.search_query.clear();
            self.search_matches.clear();
            self.search_selected = 0;
            return;
        }

        if code == KeyCode::Char('v') {
            self.open_file_viewer();
            return;
        }

        if code == KeyCode::Char('?') {
            self.open_fuzzy_finder();
            return;
        }

        if code == KeyCode::Char('!') {
            self.shell_input.clear();
            self.mode = Mode::ShellInput(String::new());
            return;
        }

        if code == KeyCode::Char('b') && modifiers == KeyModifiers::NONE {
            self.add_bookmark();
            return;
        }

        if code == KeyCode::Char('B') {
            self.bookmark_selected = 0;
            self.mode = Mode::Bookmarks;
            return;
        }

        // Command palette
        if code == KeyCode::Char(':') {
            self.mode = Mode::Command(String::new());
            return;
        }

        // Batch selection toggle with Space
        if code == KeyCode::Char(' ') {
            if self.selected_indices.contains(&self.selected) {
                self.selected_indices.remove(&self.selected);
            } else if self.selected < self.files.len() {
                self.selected_indices.insert(self.selected);
            }
            return;
        }

        // File operations
        if code == KeyCode::Char('c') || config::key_matches(&cfg.keys.copy, key) {
            self.perform_batch_operation(Operation::Copy);
            return;
        }
        if code == KeyCode::Char('m') || config::key_matches(&cfg.keys.r#move, key) {
            self.perform_batch_operation(Operation::Move);
            return;
        }
        if code == KeyCode::Char('p') {
            self.perform_paste();
            return;
        }
        if code == KeyCode::Char('d') || config::key_matches(&cfg.keys.delete, key) {
            self.perform_delete();
            return;
        }
        if (code == KeyCode::Char('r') || config::key_matches(&cfg.keys.rename, key))
            && let Some(name) = self.selected_name().cloned()
        {
            self.rename_input = name.clone();
            self.mode = Mode::Rename(name);
        }

        // Batch rename with Shift+R
        if code == KeyCode::Char('R') {
            if self.selected_indices.is_empty() && self.selected < self.files.len() {
                // No batch selection: select current item
                self.selected_indices.insert(self.selected);
            }
            if !self.selected_indices.is_empty() {
                self.batch_rename_input.clear();
                self.mode = Mode::BatchRename;
            } else {
                self.set_status("No items selected for batch rename".to_string());
            }
        }
    }

    fn perform_batch_operation(&mut self, op: Operation) {
        let indices: Vec<usize> = if self.selected_indices.is_empty() {
            vec![self.selected]
        } else {
            self.selected_indices.iter().copied().collect()
        };

        let mut items = Vec::new();
        for idx in indices {
            if let Some((name, _)) = self.files.get(idx) {
                let path = self.current_dir.join(name);
                items.push((path, op.clone()));
            }
        }

        if items.is_empty() {
            self.set_status("No items selected".to_string());
            return;
        }

        let op_name = match op {
            Operation::Copy => "Copied",
            Operation::Move => "Cut",
        };
        self.clipboard = items;
        self.selected_indices.clear();
        let count = self.clipboard.len();
        self.set_status(format!("{} {} item(s) to clipboard. Press 'p' to paste.", op_name, count));
    }

    fn perform_paste(&mut self) {
        if self.clipboard.is_empty() {
            self.set_status("Clipboard empty".to_string());
            return;
        }

        let mut overwritten = Vec::new();
        let clipboard: Vec<(PathBuf, Operation)> = self.clipboard.clone();
        for (src, op) in &clipboard {
            let name = src
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let dst = self.current_dir.join(&name);
            if dst.exists() {
                overwritten.push((src.clone(), dst, op.clone()));
            } else {
                self.execute_paste(src, &dst, op.clone());
            }
        }

        if !overwritten.is_empty() {
            let (src, dst, op) = overwritten.into_iter().next().unwrap();
            self.mode = Mode::Confirm(ConfirmAction::Overwrite(src, dst, op));
        } else {
            self.clipboard.clear();
        }
    }

    fn perform_delete(&mut self) {
        let indices: Vec<usize> = if self.selected_indices.is_empty() {
            vec![self.selected]
        } else {
            self.selected_indices.iter().copied().collect()
        };

        let paths: Vec<PathBuf> = indices
            .iter()
            .filter_map(|&idx| {
                self.files.get(idx).map(|(name, _)| self.current_dir.join(name))
            })
            .collect();

        if paths.is_empty() {
            self.set_status("No items to delete".to_string());
            return;
        }

        if paths.len() == 1 {
            self.mode = Mode::Confirm(ConfirmAction::Delete(paths.into_iter().next().unwrap()));
        } else {
            self.mode = Mode::Confirm(ConfirmAction::DeleteBatch(paths));
        }
    }

    fn perform_batch_rename(&mut self, pattern: &str) {
        let indices: Vec<usize> = self.selected_indices.iter().copied().collect();
        if indices.is_empty() {
            self.set_status("No items selected".to_string());
            return;
        }

        let count = indices.len();
        let new_names = files::generate_sequential_names(pattern, count);

        let mut renames = Vec::with_capacity(count);
        for (idx, new_name) in indices.iter().zip(new_names.into_iter()) {
            if let Some((old_name, _)) = self.files.get(*idx) {
                if !files::valid_name(&new_name) {
                    self.set_status(format!("Invalid name generated: {}", new_name));
                    return;
                }
                renames.push((old_name.clone(), new_name));
            }
        }

        match files::batch_rename(&self.current_dir, &renames) {
            Ok((success, failed)) => {
                self.set_status(format!("Renamed {} item(s), {} failed", success, failed));
                self.selected_indices.clear();
                self.refresh();
            }
            Err(e) => {
                self.set_status(files::format_err(e));
            }
        }
    }

    fn get_selected_paths(&self) -> Vec<PathBuf> {
        let indices: Vec<usize> = if self.selected_indices.is_empty() {
            if self.selected < self.files.len() {
                vec![self.selected]
            } else {
                vec![]
            }
        } else {
            self.selected_indices.iter().copied().collect()
        };
        indices.iter()
            .filter_map(|&idx| self.files.get(idx).map(|(name, _)| self.current_dir.join(name)))
            .collect()
    }

    fn execute_command(&mut self, cmd: &str) {
        let parts: Vec<&str> = cmd.trim().split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        match parts[0] {
            "quit" | "q" => std::process::exit(0),
            "reload" | "r" => {
                self.refresh();
                self.set_status("Directory reloaded".to_string());
            }
            "hidden" | "h" => {
                self.show_hidden = !self.show_hidden;
                self.refresh();
            }
            "theme" | "t" => {
                if parts.len() < 2 {
                    let names: Vec<String> = self.themes.iter().map(|(n, _)| n.clone()).collect();
                    if names.is_empty() {
                        self.set_status("No themes available".to_string());
                    } else {
                        self.set_status(format!("Available themes: {}", names.join(", ")));
                    }
                    return;
                }
                let theme_name = parts[1];
                if let Some((_, theme_config)) = self.themes.iter().find(|(n, _)| n == theme_name) {
                    self.config.colors = theme_config.colors.clone();
                    self.config.active_theme = Some(theme_name.to_string());
                    config::save_config(&self.config);
                    self.set_status(format!("Theme set to: {}", theme_name));
                } else {
                    self.set_status(format!("Theme not found: {}", theme_name));
                }
            }
            "clear" | "cls" => {
                self.selected_indices.clear();
                self.clipboard.clear();
                self.set_status("Selection and clipboard cleared".to_string());
            }
            "mkdir" => {
                if parts.len() < 2 {
                    self.set_status("Usage: :mkdir <name>".to_string());
                    return;
                }
                let name = parts[1];
                if !files::valid_name(name) {
                    self.set_status("Invalid directory name".to_string());
                    return;
                }
                let path = self.current_dir.join(name);
                match files::create_dir(&path) {
                    Ok(()) => {
                        self.set_status(format!("Created directory: {}", name));
                        self.refresh();
                    }
                    Err(e) => self.set_status(files::format_err(e)),
                }
            }
            "touch" => {
                if parts.len() < 2 {
                    self.set_status("Usage: :touch <name>".to_string());
                    return;
                }
                let name = parts[1];
                if !files::valid_name(name) {
                    self.set_status("Invalid file name".to_string());
                    return;
                }
                let path = self.current_dir.join(name);
                match files::touch_file(&path) {
                    Ok(()) => {
                        self.set_status(format!("Created file: {}", name));
                        self.refresh();
                    }
                    Err(e) => self.set_status(files::format_err(e)),
                }
            }
            "open" | "o" => {
                if let Some(path) = self.selected_path() {
                    match files::open_item(&path) {
                        Ok(()) => self.set_status(format!("Opened: {}", path.display())),
                        Err(e) => self.set_status(files::format_err(e)),
                    }
                } else {
                    self.set_status("No file selected".to_string());
                }
            }
            "sort" | "s" => {
                if parts.len() < 2 {
                    self.set_status("Usage: :sort <name|size|modified>".to_string());
                    return;
                }
                self.sort_by = match parts[1] {
                    "name" | "n" => SortBy::Name,
                    "size" | "sz" => SortBy::Size,
                    "modified" | "m" | "time" | "t" => SortBy::Modified,
                    _ => {
                        self.set_status(format!("Unknown sort mode: {}", parts[1]));
                        return;
                    }
                };
                self.refresh();
                self.set_status(format!("Sorted by: {}", parts[1]));
            }
            "filter" | "f" => {
                if parts.len() < 2 {
                    self.filter_ext = None;
                    self.refresh();
                    self.set_status("Filter cleared".to_string());
                    return;
                }
                let ext = parts[1].trim_start_matches('.');
                self.filter_ext = Some(ext.to_string());
                self.refresh();
                self.set_status(format!("Filtered by: .{}", ext));
            }
            "batchrename" | "br" => {
                if parts.len() < 2 {
                    self.set_status("Usage: :batchrename <pattern>".to_string());
                    return;
                }
                let pattern = parts[1];
                if self.selected_indices.is_empty() && self.selected < self.files.len() {
                    self.selected_indices.insert(self.selected);
                }
                self.perform_batch_rename(pattern);
            }
            "zip" => {
                if parts.len() < 2 {
                    self.set_status("Usage: :zip <name>".to_string());
                    return;
                }
                let name = format!("{}.zip", parts[1].trim_end_matches(".zip"));
                let paths = self.get_selected_paths();
                if paths.is_empty() {
                    self.set_status("No items selected".to_string());
                    return;
                }
                let output = self.current_dir.join(&name);
                match files::create_zip_archive(&paths, &output) {
                    Ok(()) => {
                        self.set_status(format!("Created archive: {}", name));
                        self.refresh();
                    }
                    Err(e) => self.set_status(files::format_err(e)),
                }
            }
            "tar" => {
                if parts.len() < 2 {
                    self.set_status("Usage: :tar <name>".to_string());
                    return;
                }
                let name = format!("{}.tar.gz", parts[1].trim_end_matches(".tar.gz"));
                let paths = self.get_selected_paths();
                if paths.is_empty() {
                    self.set_status("No items selected".to_string());
                    return;
                }
                let output = self.current_dir.join(&name);
                match files::create_tar_archive(&paths, &output) {
                    Ok(()) => {
                        self.set_status(format!("Created archive: {}", name));
                        self.refresh();
                    }
                    Err(e) => self.set_status(files::format_err(e)),
                }
            }
            "extract" | "x" => {
                if let Some(path) = self.selected_path() {
                    if !files::is_archive_file(&path) {
                        self.set_status("Selected file is not an archive".to_string());
                        return;
                    }
                    let output_dir = self.current_dir.join(
                        path.file_stem().unwrap_or(path.as_os_str())
                    );
                    match files::extract_archive(&path, &output_dir) {
                        Ok(()) => {
                            self.set_status(format!("Extracted to: {}", output_dir.display()));
                            self.refresh();
                        }
                        Err(e) => self.set_status(files::format_err(e)),
                    }
                } else {
                    self.set_status("No archive selected".to_string());
                }
            }
            "find" => {
                self.open_fuzzy_finder();
            }
            "shell" | "sh" => {
                let cmd = parts[1..].join(" ");
                self.run_shell_command(&cmd);
            }
            "bookmark" | "bm" => {
                self.bookmark_selected = 0;
                self.mode = Mode::Bookmarks;
            }
            "bookmark-add" | "ba" => {
                self.add_bookmark();
            }
            _ => {
                self.set_status(format!("Unknown command: {}", parts[0]));
            }
        }
    }

    fn update_search(&mut self) {
        let matcher = SkimMatcherV2::default();
        let query = self.search_query.to_lowercase();
        self.search_matches = self
            .files
            .iter()
            .enumerate()
            .filter_map(|(i, (name, _))| {
                matcher
                    .fuzzy_match(&name.to_lowercase(), &query)
                    .map(|_| i)
            })
            .collect();
        self.search_selected = 0;
    }

    fn perform_rename(&mut self, old: &str, new: &str) {
        if new == old {
            return;
        }
        if !files::valid_name(new) {
            self.set_status("Invalid name".to_string());
            return;
        }
        if let Err(e) = files::rename_item(&self.current_dir, old, new) {
            self.set_status(files::format_err(e));
        } else {
            self.set_status(format!("Renamed '{}' -> '{}'", old, new));
            self.refresh();
        }
    }

    fn execute_confirm(&mut self, action: ConfirmAction) {
        match action {
            ConfirmAction::Delete(path) => {
                if let Err(e) = files::delete_item(&path) {
                    self.set_status(files::format_err(e));
                } else {
                    self.set_status(format!("Deleted {}", path.display()));
                    self.refresh();
                }
            }
            ConfirmAction::DeleteBatch(paths) => {
                let total = paths.len();
                let mut success = 0;
                let mut failed = 0;
                for (i, path) in paths.iter().enumerate() {
                    if let Err(_) = files::delete_item(path) {
                        failed += 1;
                    } else {
                        success += 1;
                    }
                    // Update progress every few items
                    if i % 5 == 0 || i == total - 1 {
                        self.set_status(format!("Deleting... {}/{}", i + 1, total));
                    }
                }
                self.set_status(format!("Deleted {} item(s), {} failed", success, failed));
                self.selected_indices.clear();
                self.refresh();
            }
            ConfirmAction::Overwrite(src, dst, op) => {
                self.execute_paste(&src, &dst, op);
            }
        }
    }

    fn execute_paste(&mut self, src: &Path, dst: &Path, op: Operation) {
        match op {
            Operation::Copy => {
                if let Err(e) = files::copy_item(src, dst) {
                    self.set_status(files::format_err(e));
                } else {
                    self.set_status(format!("Copied {}", src.display()));
                    self.refresh();
                }
            }
            Operation::Move => {
                if let Err(e) = files::move_item(src, dst) {
                    self.set_status(files::format_err(e));
                } else {
                    self.set_status(format!("Moved {}", src.display()));
                    self.refresh();
                }
            }
        }
    }

    fn draw(&mut self, f: &mut ratatui::Frame) {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(f.area());

        let body_area = main_chunks[0];
        let status_area = main_chunks[1];

        let body_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(body_area);

        // Update list height for page scrolling
        self.list_height = body_chunks[0].height.saturating_sub(2);

        let dir_color = parse_color(&self.config.colors.directory);
        let file_color = parse_color(&self.config.colors.file);
        let border_color = parse_color(&self.config.colors.border);
        let highlight = config::highlight_style(&self.config);
        let select_color = Color::Rgb(255, 105, 180); // hot pink for batch selection

        // File list
        let items: Vec<ListItem> = self
            .files
            .iter()
            .enumerate()
            .map(|(i, (name, is_dir))| {
                let is_match = if let Mode::Search = self.mode {
                    self.search_matches.get(self.search_selected) == Some(&i)
                        && !self.search_query.is_empty()
                } else {
                    false
                };

                let is_selected = self.selected_indices.contains(&i);

                let style = if i == self.selected || is_match {
                    highlight
                } else if is_selected {
                    Style::default().fg(select_color).add_modifier(Modifier::BOLD)
                } else if *is_dir {
                    Style::default().fg(dir_color)
                } else {
                    Style::default().fg(file_color)
                };

                let prefix = if is_selected {
                    "▸ "
                } else if *is_dir {
                    "📁 "
                } else {
                    "📄 "
                };

                ListItem::new(format!("{}{}", prefix, name)).style(style)
            })
            .collect();

        let list_title = match &self.mode {
            Mode::Search => format!("VHS-86 — search: {}", self.search_query),
            Mode::Command(cmd) => format!("VHS-86 — :{}", cmd),
            Mode::ShellInput(cmd) => format!("VHS-86 — shell: {}", cmd),
            Mode::FuzzyFinder(query) => format!("VHS-86 — find: {}", query),
            _ => {
                let selected_count = self.selected_indices.len();
                let filter_info = self.filter_ext.as_ref()
                    .map(|e| format!(" | filter: .{}", e))
                    .unwrap_or_default();
                if selected_count > 0 {
                    format!("VHS-86 [{} selected]{}", selected_count, filter_info)
                } else {
                    format!("VHS-86{}", filter_info)
                }
            }
        };

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(list_title)
                .border_style(Style::default().fg(border_color)),
        );
        f.render_widget(list, body_chunks[0]);

        // Preview pane
        let preview_area = body_chunks[1];
        let preview_block = Block::default()
            .borders(Borders::ALL)
            .title("Preview")
            .border_style(Style::default().fg(border_color));

        // Try to render image preview first
        let mut image_rendered = false;
        if let Some(ref mut state) = self.image_state {
            let inner = preview_block.inner(preview_area);
            if inner.width > 0 && inner.height > 0 {
                let img_widget = StatefulImage::default();
                f.render_stateful_widget(img_widget, inner, state);
                let _ = state.last_encoding_result();
                image_rendered = true;
            }
        }

        if !image_rendered {
            let preview = if let Some((name, is_dir)) = self.files.get(self.selected) {
                let path = self.current_dir.join(name);
                if *is_dir {
                    Paragraph::new(format!("Directory: {}", path.display()))
                } else if files::is_image_file(&path) {
                    match image::image_dimensions(&path) {
                        Ok((w, h)) => Paragraph::new(format!(
                            "🖼️  Image: {}\nDimensions: {}x{} pixels",
                            name,
                            w,
                            h
                        )),
                        Err(_) => Paragraph::new(format!("🖼️  Image: {} (unable to read dimensions)", name)),
                    }
                } else {
                    match std::fs::read_to_string(&path) {
                        Ok(content) => Paragraph::new(content.chars().take(2000).collect::<String>()),
                        Err(_) => Paragraph::new("Binary file"),
                    }
                }
            } else {
                Paragraph::new("Empty")
            };
            let preview = preview.block(preview_block);
            f.render_widget(preview, preview_area);
        } else {
            // Render empty block for image (image was rendered directly)
            f.render_widget(preview_block, preview_area);
        }

        // Status line
        let status_text = if let Some((msg, _)) = &self.status_message {
            msg.clone()
        } else {
            let selected_info = if !self.selected_indices.is_empty() {
                format!(" | {} selected", self.selected_indices.len())
            } else {
                String::new()
            };
            let sort_info = format!(" | sort: {:?}", self.sort_by).to_ascii_lowercase();
            format!(
                "{} | {} items{}{} | hidden: {}",
                self.current_dir.display(),
                self.files.len(),
                selected_info,
                sort_info,
                if self.show_hidden { "on" } else { "off" }
            )
        };
        let status_color = parse_color(&self.config.colors.status);
        let status = Paragraph::new(status_text)
            .style(Style::default().fg(status_color).add_modifier(Modifier::BOLD));
        f.render_widget(status, status_area);

        // Modal overlays
        match &self.mode {
            Mode::Rename(_) => self.draw_input_box(f, "Rename", &self.rename_input),
            Mode::BatchRename => self.draw_input_box(f, "Batch Rename (pattern: vacation_{:03}.jpg)", &self.batch_rename_input),
            Mode::ShellInput(input) => self.draw_input_box(f, "Shell Command", input),
            Mode::ShellOutput(output) => self.draw_shell_output(f, output),
            Mode::FileViewer => self.draw_file_viewer(f),
            Mode::FuzzyFinder(_) => self.draw_fuzzy_finder(f),
            Mode::Bookmarks => self.draw_bookmarks(f),
            Mode::Confirm(ConfirmAction::Delete(path)) => {
                self.draw_confirm_box(f, &format!("Delete {}? [y/N]", path.display()))
            }
            Mode::Confirm(ConfirmAction::DeleteBatch(paths)) => {
                self.draw_confirm_box(f, &format!("Delete {} items? [y/N]", paths.len()))
            }
            Mode::Confirm(ConfirmAction::Overwrite(_, dst, _)) => {
                self.draw_confirm_box(f, &format!("Overwrite {}? [y/N]", dst.display()))
            }
            Mode::Command(cmd) => {
                self.draw_command_box(f, cmd);
            }
            _ => {}
        }
    }

    fn draw_input_box(&self, f: &mut ratatui::Frame, title: &str, content: &str) {
        let area = centered_rect(50, 20, f.area());
        f.render_widget(Clear, area);
        let input = Paragraph::new(content.to_string())
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(Color::Magenta)),
            );
        f.render_widget(input, area);
    }

    fn draw_confirm_box(&self, f: &mut ratatui::Frame, message: &str) {
        let area = centered_rect(60, 20, f.area());
        f.render_widget(Clear, area);
        let confirm = Paragraph::new(message.to_string())
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Confirm")
                    .border_style(Style::default().fg(Color::Red)),
            );
        f.render_widget(confirm, area);
    }

    fn draw_command_box(&self, f: &mut ratatui::Frame, cmd: &str) {
        let area = centered_rect(70, 25, f.area());
        f.render_widget(Clear, area);

        let help_text = format!(
            "Command: {}\n\nAvailable commands:\n\
             \u{0020} :quit / :q          — exit\n\
             \u{0020} :reload / :r        — reload directory\n\
             \u{0020} :hidden / :h        — toggle hidden files\n\
             \u{0020} :theme <name>       — switch theme\n\
             \u{0020} :clear              — clear selection\n\
             \u{0020} :mkdir <name>       — create directory\n\
             \u{0020} :touch <name>       — create empty file\n\
             \u{0020} :open / :o          — open selected file\n\
             \u{0020} :sort <name|size|modified> — sort files\n\
             \u{0020} :filter <ext>       — filter by extension\n\
             \u{0020} :batchrename <pat>  — batch rename selected\n\
             \u{0020} :zip <name>         — create zip archive\n\
             \u{0020} :tar <name>         — create tar.gz archive\n\
             \u{0020} :extract / :x       — extract selected archive\n\
             \u{0020} :find / :f          — recursive fuzzy finder\n\
             \u{0020} :shell <cmd>        — run shell command\n\
             \u{0020} :bookmark / :bm     — list bookmarks\n\
             \u{0020} :bookmark-add / :ba — add current dir to bookmarks",
            cmd
        );

        let command_widget = Paragraph::new(help_text)
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Command Palette")
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        f.render_widget(command_widget, area);
    }

    fn draw_shell_output(&self, f: &mut ratatui::Frame, output: &str) {
        let area = centered_rect(85, 70, f.area());
        f.render_widget(Clear, area);

        let lines: Vec<&str> = output.lines().collect();
        let visible: Vec<String> = lines.iter()
            .skip(self.shell_scroll)
            .take(area.height.saturating_sub(2) as usize)
            .map(|l| l.to_string())
            .collect();

        let content = visible.join("\n");
        let paragraph = Paragraph::new(content)
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Shell Output ({} lines, scroll: {})", lines.len(), self.shell_scroll))
                    .border_style(Style::default().fg(Color::Green)),
            );
        f.render_widget(paragraph, area);
    }

    fn draw_file_viewer(&self, f: &mut ratatui::Frame) {
        let area = centered_rect(90, 85, f.area());
        f.render_widget(Clear, area);

        let visible: Vec<ratatui::text::Line> = self.file_viewer_content.iter()
            .skip(self.file_viewer_scroll)
            .take(area.height.saturating_sub(2) as usize)
            .map(|(text, style)| {
                ratatui::text::Line::styled(text.clone(), *style)
            })
            .collect();

        let paragraph = Paragraph::new(ratatui::text::Text::from(visible))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("{} ({} lines, scroll: {})", self.file_viewer_title, self.file_viewer_content.len(), self.file_viewer_scroll))
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        f.render_widget(paragraph, area);
    }

    fn draw_fuzzy_finder(&self, f: &mut ratatui::Frame) {
        let area = centered_rect(80, 70, f.area());
        f.render_widget(Clear, area);

        let query = match &self.mode {
            Mode::FuzzyFinder(q) => q.clone(),
            _ => String::new(),
        };

        let items: Vec<ListItem> = self.fuzzy_finder_matches.iter()
            .enumerate()
            .map(|(i, &idx)| {
                let path = &self.fuzzy_finder_paths[idx];
                let display = path.strip_prefix(&self.current_dir)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .to_string();
                let style = if i == self.fuzzy_finder_selected {
                    Style::default().bg(Color::Magenta).fg(Color::Black).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(display).style(style)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Fuzzy Finder: {} ({} matches)", query, self.fuzzy_finder_matches.len()))
                .border_style(Style::default().fg(Color::Yellow)),
        );
        f.render_widget(list, area);
    }

    fn draw_bookmarks(&self, f: &mut ratatui::Frame) {
        let area = centered_rect(60, 50, f.area());
        f.render_widget(Clear, area);

        if self.config.bookmarks.is_empty() {
            let msg = Paragraph::new("No bookmarks saved.\nPress 'b' in normal mode to bookmark current directory.")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Bookmarks")
                        .border_style(Style::default().fg(Color::Cyan)),
                );
            f.render_widget(msg, area);
            return;
        }

        let items: Vec<ListItem> = self.config.bookmarks.iter()
            .enumerate()
            .map(|(i, path)| {
                let style = if i == self.bookmark_selected {
                    Style::default().bg(Color::Magenta).fg(Color::Black).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(format!("{} {}", i + 1, path)).style(style)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Bookmarks ({} saved)", self.config.bookmarks.len()))
                .border_style(Style::default().fg(Color::Cyan)),
        );
        f.render_widget(list, area);
    }
}

/// Compute a centered rectangle with the given percent width/height.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn setup_config_watcher() -> Option<mpsc::Receiver<()>> {
    let (tx, rx) = mpsc::channel();
    if let Some(path) = config::config_path() {
        if let Some(parent) = path.parent() {
            let parent = parent.to_path_buf();
            let tx = tx.clone();
            if let Ok(mut watcher) = RecommendedWatcher::new(
                move |res: Result<NotifyEvent, notify::Error>| {
                    if let Ok(event) = res {
                        if event.kind.is_modify() {
                            let _ = tx.send(());
                        }
                    }
                },
                NotifyConfig::default(),
            ) {
                let _ = watcher.watch(&parent, RecursiveMode::NonRecursive);
                std::mem::forget(watcher);
            }
        }
    }
    Some(rx)
}

fn main() -> io::Result<()> {
    let mut terminal = setup_terminal()?;

    let initial_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));

    let config = config::load_config();
    let mut app = App::new(initial_dir, config);

    loop {
        app.clear_status_if_expired();
        app.check_config_reload();
        app.update_image_preview();

        terminal.draw(|f| app.draw(f))?;

        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            app.handle_key(key.code, key.modifiers);
        }
    }
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

#[allow(dead_code)]
fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
