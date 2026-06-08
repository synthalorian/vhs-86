use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Synthwave-inspired default color scheme for VHS-86.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Colors {
    pub background: String,
    pub foreground: String,
    pub highlight_bg: String,
    pub highlight_fg: String,
    pub directory: String,
    pub file: String,
    pub border: String,
    pub status: String,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            background: "black".to_string(),
            foreground: "white".to_string(),
            highlight_bg: "magenta".to_string(),
            highlight_fg: "black".to_string(),
            directory: "cyan".to_string(),
            file: "white".to_string(),
            border: "magenta".to_string(),
            status: "yellow".to_string(),
        }
    }
}

/// User-configurable keybindings. Missing keys fall back to defaults.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyBindings {
    pub quit: Option<String>,
    pub up: Option<String>,
    pub down: Option<String>,
    pub left: Option<String>,
    pub right: Option<String>,
    pub top: Option<String>,
    pub bottom: Option<String>,
    pub home: Option<String>,
    pub toggle_hidden: Option<String>,
    pub copy: Option<String>,
    pub r#move: Option<String>,
    pub delete: Option<String>,
    pub rename: Option<String>,
    pub search: Option<String>,
    pub confirm: Option<String>,
    pub cancel: Option<String>,
}

/// Full application configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub colors: Colors,
    pub keys: KeyBindings,
    pub active_theme: Option<String>,
    pub bookmarks: Vec<String>,
}

/// Resolve the default config directory and file path.
pub fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("vhs-86").join("config.toml"))
}

/// Resolve the themes directory path.
pub fn themes_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("vhs-86").join("themes"))
}

/// Load config from disk, or create a default config file if none exists.
pub fn load_config() -> Config {
    match config_path() {
        Some(path) => {
            if path.exists() {
                match fs::read_to_string(&path) {
                    Ok(content) => {
                        let mut config: Config = toml::from_str(&content).unwrap_or_default();
                        // Apply active theme if set
                        if let Some(ref theme_name) = config.active_theme {
                            let themes = load_themes();
                            if let Some((_, theme_cfg)) = themes.iter().find(|(n, _)| n == theme_name) {
                                config.colors = theme_cfg.colors.clone();
                            }
                        }
                        config
                    }
                    Err(_) => Config::default(),
                }
            } else {
                let default = Config::default();
                if let Some(parent) = path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(&path, default.to_toml());
                default
            }
        }
        None => Config::default(),
    }
}

/// Save config to disk, preserving user settings.
pub fn save_config(config: &Config) {
    if let Some(path) = config_path() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&path, config.to_toml());
    }
}

/// Built-in themes shipped with VHS-86.
pub fn built_in_themes() -> Vec<(String, Config)> {
    vec![
        (
            "synthwave-84".to_string(),
            Config {
                colors: Colors {
                    background: "black".to_string(),
                    foreground: "white".to_string(),
                    highlight_bg: "magenta".to_string(),
                    highlight_fg: "black".to_string(),
                    directory: "cyan".to_string(),
                    file: "white".to_string(),
                    border: "magenta".to_string(),
                    status: "yellow".to_string(),
                },
                keys: KeyBindings::default(),
                active_theme: Some("synthwave-84".to_string()),
                bookmarks: Vec::new(),
            },
        ),
        (
            "midnight-green".to_string(),
            Config {
                colors: Colors {
                    background: "black".to_string(),
                    foreground: "rgb(200,220,200)".to_string(),
                    highlight_bg: "rgb(0,128,128)".to_string(),
                    highlight_fg: "black".to_string(),
                    directory: "rgb(0,200,150)".to_string(),
                    file: "rgb(200,220,200)".to_string(),
                    border: "rgb(0,128,128)".to_string(),
                    status: "rgb(100,200,100)".to_string(),
                },
                keys: KeyBindings::default(),
                active_theme: Some("midnight-green".to_string()),
                bookmarks: Vec::new(),
            },
        ),
        (
            "amber-crt".to_string(),
            Config {
                colors: Colors {
                    background: "black".to_string(),
                    foreground: "rgb(255,176,0)".to_string(),
                    highlight_bg: "rgb(255,176,0)".to_string(),
                    highlight_fg: "black".to_string(),
                    directory: "rgb(255,200,50)".to_string(),
                    file: "rgb(255,176,0)".to_string(),
                    border: "rgb(200,140,0)".to_string(),
                    status: "rgb(255,220,80)".to_string(),
                },
                keys: KeyBindings::default(),
                active_theme: Some("amber-crt".to_string()),
                bookmarks: Vec::new(),
            },
        ),
    ]
}

/// Load all available themes from ~/.config/vhs-86/themes/*.toml plus built-ins.
/// Returns a vector of (theme_name, config_with_colors) pairs.
pub fn load_themes() -> Vec<(String, Config)> {
    let mut themes = built_in_themes();

    let dir = match themes_dir() {
        Some(d) => d,
        None => return themes,
    };

    if !dir.exists() {
        // Create themes directory with a sample theme file
        let _ = fs::create_dir_all(&dir);
        let sample_path = dir.join("neon-dream.toml");
        let neon = r#"# Neon Dream theme for VHS-86
[colors]
background = "black"
foreground = "rgb(200,200,255)"
highlight_bg = "rgb(255,0,255)"
highlight_fg = "black"
directory = "rgb(0,255,255)"
file = "rgb(200,200,255)"
border = "rgb(255,0,255)"
status = "rgb(255,255,0)"
"#;
        let _ = fs::write(&sample_path, neon);
    }

    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return themes,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map(|e| e == "toml").unwrap_or(false) {
            let name = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            // Skip if a built-in theme with the same name already exists
            if themes.iter().any(|(n, _)| n == &name) {
                continue;
            }

            if let Ok(content) = fs::read_to_string(&path) {
                // Try to parse just the colors section
                if let Ok(config) = toml::from_str::<Config>(&content) {
                    themes.push((name, config));
                }
            }
        }
    }

    themes
}

impl Config {
    /// Serialize the config to a TOML string with helpful comments.
    pub fn to_toml(&self) -> String {
        let theme_line = match &self.active_theme {
            Some(t) => format!("active_theme = \"{}\"\n", t),
            None => "# active_theme = \"synthwave-84\"\n".to_string(),
        };
        let bookmarks_toml = if self.bookmarks.is_empty() {
            "# bookmarks = [\n#     \"/home/user/projects\",\n#     \"/home/user/documents\",\n# ]\n".to_string()
        } else {
            let entries: Vec<String> = self.bookmarks.iter()
                .map(|b| format!("    \"{}\"", b))
                .collect();
            format!("bookmarks = [\n{}\n]\n", entries.join(",\n"))
        };
        format!(
            "# VHS-86 configuration\n\
             # Colors: black, red, green, yellow, blue, magenta, cyan, gray,\n\
             #         dark_gray, light_red, light_green, light_yellow,\n\
             #         light_blue, light_magenta, light_cyan, white,\n\
             #         or rgb(r,g,b) e.g. rgb(255,0,255)\n\
             [colors]\n\
             background = \"{}\"\n\
             foreground = \"{}\"\n\
             highlight_bg = \"{}\"\n\
             highlight_fg = \"{}\"\n\
             directory = \"{}\"\n\
             file = \"{}\"\n\
             border = \"{}\"\n\
             status = \"{}\"\n\n\
             # Active theme name. Built-in: synthwave-84, midnight-green, amber-crt\n\
             {}\n\
             # Saved directory bookmarks for quick jumping\n\
             {}\n\
             # Optional keybinding overrides.\n\
             # Use a single character or special name: enter, esc, backspace.\n\
             [keys]\n\
             # quit = \"q\"\n\
             # up = \"k\"\n\
             # down = \"j\"\n\
             # left = \"h\"\n\
             # right = \"l\"\n\
             # top = \"g\"\n\
             # bottom = \"G\"\n\
             # home = \"~\"\n\
             # toggle_hidden = \".\"\n\
             # copy = \"c\"\n\
             # move = \"m\"\n\
             # delete = \"d\"\n\
             # rename = \"r\"\n\
             # search = \"/\"\n\
             # confirm = \"y\"\n\
             # cancel = \"n\"\n",
            self.colors.background,
            self.colors.foreground,
            self.colors.highlight_bg,
            self.colors.highlight_fg,
            self.colors.directory,
            self.colors.file,
            self.colors.border,
            self.colors.status,
            theme_line,
            bookmarks_toml,
        )
    }
}

/// Parse common color names plus `rgb(r,g,b)` syntax.
pub fn parse_color(s: &str) -> Color {
    let s = s.trim().to_ascii_lowercase();
    if let Some(inner) = s.strip_prefix("rgb(").and_then(|x| x.strip_suffix(")")) {
        let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
        if parts.len() == 3
        && let (Ok(r), Ok(g), Ok(b)) = (
            parts[0].parse::<u8>(),
            parts[1].parse::<u8>(),
            parts[2].parse::<u8>(),
        )
    {
        return Color::Rgb(r, g, b);
    }
    }
    match s.as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "gray" | "grey" => Color::Gray,
        "dark_gray" | "dark_grey" => Color::DarkGray,
        "light_red" => Color::LightRed,
        "light_green" => Color::LightGreen,
        "light_yellow" => Color::LightYellow,
        "light_blue" => Color::LightBlue,
        "light_magenta" => Color::LightMagenta,
        "light_cyan" => Color::LightCyan,
        "white" => Color::White,
        _ => Color::White,
    }
}

/// Match a key string from config against a typed character.
pub fn key_matches(configured: &Option<String>, code: &KeyCodeChar) -> bool {
    match (configured, code) {
        (Some(k), KeyCodeChar::Char(c)) => k == c,
        (Some(k), KeyCodeChar::Enter) => k == "enter",
        (Some(k), KeyCodeChar::Esc) => k == "esc",
        (Some(k), KeyCodeChar::Backspace) => k == "backspace",
        _ => false,
    }
}

/// Simplified key code representation for config matching.
#[derive(Debug, Clone, PartialEq)]
pub enum KeyCodeChar {
    Char(String),
    Enter,
    Esc,
    Backspace,
    Other,
}

impl From<crossterm::event::KeyCode> for KeyCodeChar {
    fn from(code: crossterm::event::KeyCode) -> Self {
        match code {
            crossterm::event::KeyCode::Char(c) => KeyCodeChar::Char(c.to_string()),
            crossterm::event::KeyCode::Enter => KeyCodeChar::Enter,
            crossterm::event::KeyCode::Esc => KeyCodeChar::Esc,
            crossterm::event::KeyCode::Backspace => KeyCodeChar::Backspace,
            _ => KeyCodeChar::Other,
        }
    }
}

/// Helper to get the highlight style from config.
pub fn highlight_style(cfg: &Config) -> ratatui::style::Style {
    use ratatui::style::{Modifier, Style};
    Style::default()
        .bg(parse_color(&cfg.colors.highlight_bg))
        .fg(parse_color(&cfg.colors.highlight_fg))
        .add_modifier(Modifier::BOLD)
}
