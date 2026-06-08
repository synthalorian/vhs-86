use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A loaded WASM plugin using Extism
#[derive(Debug)]
pub struct Plugin {
    pub name: String,
    pub description: String,
    _manifest: extism::Manifest,
    _plugin: extism::Plugin,
}

/// Plugin manager that loads and runs WASM plugins
#[derive(Debug)]
pub struct PluginManager {
    plugins: HashMap<String, Plugin>,
    plugin_dir: PathBuf,
}

impl PluginManager {
    pub fn new() -> Self {
        let plugin_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("vhs-86")
            .join("plugins");

        Self {
            plugins: HashMap::new(),
            plugin_dir,
        }
    }

    /// Load all .wasm plugins from the plugin directory
    pub fn load_plugins(&mut self) {
        self.plugins.clear();

        if !self.plugin_dir.exists() {
            return;
        }

        let entries = match std::fs::read_dir(&self.plugin_dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("wasm") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Ok(plugin) = self.load_wasm_plugin(name, &path) {
                        self.plugins.insert(name.to_string(), plugin);
                    }
                }
            }
        }
    }

    fn load_wasm_plugin(&self, name: &str, path: &Path) -> Result<Plugin, Box<dyn std::error::Error>> {
        let wasm = std::fs::read(path)?;
        let manifest = extism::Manifest::new([wasm]);
        let mut plugin = extism::Plugin::new(&manifest, [], true)?;

        // Try to get plugin metadata via a "describe" function if it exists
        let description = plugin.call::<(), String>("describe", ())
            .unwrap_or_else(|_| format!("WASM plugin: {}", name));

        Ok(Plugin {
            name: name.to_string(),
            description,
            _manifest: manifest,
            _plugin: plugin,
        })
    }

    pub fn get_plugin(&self, name: &str) -> Option<&Plugin> {
        self.plugins.get(name)
    }

    pub fn list_plugins(&self) -> Vec<&Plugin> {
        self.plugins.values().collect()
    }

    pub fn count(&self) -> usize {
        self.plugins.len()
    }

    /// Run a plugin function with input data
    pub fn run(&self, name: &str, func: &str, input: &str) -> Option<String> {
        let _plugin = self.plugins.get(name)?;
        // Recreate plugin to allow multiple calls (extism plugins are consumed on call)
        let wasm = std::fs::read(&self.plugin_dir.join(format!("{}.wasm", name))).ok()?;
        let manifest = extism::Manifest::new([wasm]);
        let mut p = extism::Plugin::new(&manifest, [], true).ok()?;
        p.call::<String, String>(func, input.to_string()).ok()
    }

    /// Run preview hook on a file path
    pub fn preview_hook(&self, path: &Path) -> Option<String> {
        for plugin in self.list_plugins() {
            let wasm = std::fs::read(&self.plugin_dir.join(format!("{}.wasm", plugin.name))).ok()?;
            let manifest = extism::Manifest::new([wasm]);
            let mut p = extism::Plugin::new(&manifest, [], true).ok()?;
            let input = path.to_string_lossy().to_string();
            if let Ok(result) = p.call::<String, String>("preview", input) {
                if !result.is_empty() {
                    return Some(result);
                }
            }
        }
        None
    }

    /// Run on_file_select hook
    pub fn on_file_select(&self, path: &Path) -> Option<String> {
        for plugin in self.list_plugins() {
            let wasm = std::fs::read(&self.plugin_dir.join(format!("{}.wasm", plugin.name))).ok()?;
            let manifest = extism::Manifest::new([wasm]);
            let mut p = extism::Plugin::new(&manifest, [], true).ok()?;
            let input = path.to_string_lossy().to_string();
            if let Ok(result) = p.call::<String, String>("on_file_select", input) {
                if !result.is_empty() {
                    return Some(result);
                }
            }
        }
        None
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager_new() {
        let pm = PluginManager::new();
        assert_eq!(pm.count(), 0);
        assert!(pm.list_plugins().is_empty());
    }

    #[test]
    fn test_plugin_manager_default() {
        let pm: PluginManager = Default::default();
        assert_eq!(pm.count(), 0);
    }

    #[test]
    fn test_plugin_manager_get_nonexistent() {
        let pm = PluginManager::new();
        assert!(pm.get_plugin("nonexistent").is_none());
    }

    #[test]
    fn test_plugin_manager_load_empty_dir() {
        let mut pm = PluginManager::new();
        pm.load_plugins();
        assert_eq!(pm.count(), 0);
    }

    #[test]
    fn test_plugin_manager_preview_hook_no_plugins() {
        let pm = PluginManager::new();
        let result = pm.preview_hook(Path::new("/tmp/test"));
        assert!(result.is_none());
    }

    #[test]
    fn test_plugin_manager_on_file_select_no_plugins() {
        let pm = PluginManager::new();
        let result = pm.on_file_select(Path::new("/tmp/test"));
        assert!(result.is_none());
    }

    #[test]
    fn test_plugin_manager_run_no_plugins() {
        let pm = PluginManager::new();
        let result = pm.run("test", "func", "input");
        assert!(result.is_none());
    }
}
