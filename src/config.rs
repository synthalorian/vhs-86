use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub theme: String,
    #[serde(default = "default_show_hidden")]
    pub show_hidden: bool,
    #[serde(default = "default_preview")]
    pub preview: PreviewConfig,
    #[serde(default = "default_shell")]
    pub shell: ShellConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewConfig {
    #[serde(default = "default_true")]
    pub syntax_highlight: bool,
    #[serde(default = "default_true")]
    pub image_preview: bool,
    #[serde(default = "default_max_lines")]
    pub max_lines: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellConfig {
    #[serde(default = "default_true")]
    pub cd_on_quit: bool,
}

fn default_show_hidden() -> bool { false }
fn default_true() -> bool { true }
fn default_max_lines() -> usize { 100 }

fn default_preview() -> PreviewConfig {
    PreviewConfig {
        syntax_highlight: true,
        image_preview: true,
        max_lines: 100,
    }
}

fn default_shell() -> ShellConfig {
    ShellConfig {
        cd_on_quit: true,
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "synthwave".to_string(),
            show_hidden: false,
            preview: default_preview(),
            shell: default_shell(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(config) = toml::from_str::<Config>(&content) {
                    return config;
                }
            }
        }
        Config::default()
    }

    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("vhs-86")
            .join("config.toml")
    }
}
