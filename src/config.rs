use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::keybindings::Keybindings;

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
    #[serde(default)]
    pub keybindings: Keybindings,
    #[serde(default = "default_plugins")]
    pub plugins: PluginConfig,
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
    #[serde(default = "default_shell_command")]
    pub shell_command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub auto_load: bool,
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
        shell_command: default_shell_command(),
    }
}

fn default_shell_command() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
}

fn default_plugins() -> PluginConfig {
    PluginConfig {
        enabled: true,
        auto_load: true,
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "synthwave".to_string(),
            show_hidden: false,
            preview: default_preview(),
            shell: default_shell(),
            keybindings: Keybindings::default(),
            plugins: default_plugins(),
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
