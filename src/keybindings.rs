use crossterm::event::KeyCode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Available actions that can be bound to keys
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Quit,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    GoHome,
    GoTop,
    GoBottom,
    ToggleHidden,
    ToggleSelect,
    Enter,
    OpenChmod,
    OpenDiskUsage,
    OpenRemoteConnect,
    OpenSearch,
    OpenShell,
    BatchDelete,
    BatchCopy,
    BatchMove,
    Refresh,
}

/// A keybinding configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Keybindings {
    #[serde(flatten)]
    pub bindings: HashMap<String, Action>,
}

impl Keybindings {
    /// Load default keybindings
    pub fn defaults() -> HashMap<String, Action> {
        let mut map = HashMap::new();
        map.insert("q".to_string(), Action::Quit);
        map.insert("j".to_string(), Action::MoveDown);
        map.insert("k".to_string(), Action::MoveUp);
        map.insert("h".to_string(), Action::MoveLeft);
        map.insert("l".to_string(), Action::MoveRight);
        map.insert("down".to_string(), Action::MoveDown);
        map.insert("up".to_string(), Action::MoveUp);
        map.insert("left".to_string(), Action::MoveLeft);
        map.insert("right".to_string(), Action::MoveRight);
        map.insert("enter".to_string(), Action::Enter);
        map.insert("g".to_string(), Action::GoTop);
        map.insert("G".to_string(), Action::GoBottom);
        map.insert("~".to_string(), Action::GoHome);
        map.insert(".".to_string(), Action::ToggleHidden);
        map.insert(" ".to_string(), Action::ToggleSelect);
        map.insert("c".to_string(), Action::OpenChmod);
        map.insert("d".to_string(), Action::OpenDiskUsage);
        map.insert("r".to_string(), Action::OpenRemoteConnect);
        map.insert("/".to_string(), Action::OpenSearch);
        map.insert("!".to_string(), Action::OpenShell);
        map.insert("D".to_string(), Action::BatchDelete);
        map.insert("C".to_string(), Action::BatchCopy);
        map.insert("M".to_string(), Action::BatchMove);
        map.insert("R".to_string(), Action::Refresh);
        map
    }

    /// Get the action for a key code, falling back to defaults
    pub fn get_action(&self, code: &KeyCode) -> Option<Action> {
        let key_str = keycode_to_string(code);
        self.bindings.get(&key_str).copied().or_else(|| {
            // Fall back to defaults if not in custom bindings
            Self::defaults().get(&key_str).copied()
        })
    }

    /// Convert a KeyCode to a string representation for config matching
    pub fn keycode_to_string(code: &KeyCode) -> String {
        match code {
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Up => "up".to_string(),
            KeyCode::Down => "down".to_string(),
            KeyCode::Left => "left".to_string(),
            KeyCode::Right => "right".to_string(),
            KeyCode::Enter => "enter".to_string(),
            KeyCode::Esc => "esc".to_string(),
            KeyCode::Backspace => "backspace".to_string(),
            KeyCode::Tab => "tab".to_string(),
            KeyCode::Home => "home".to_string(),
            KeyCode::End => "end".to_string(),
            KeyCode::PageUp => "pageup".to_string(),
            KeyCode::PageDown => "pagedown".to_string(),
            _ => format!("{:?}", code).to_lowercase(),
        }
    }
}

/// Helper to convert KeyCode to string
pub fn keycode_to_string(code: &KeyCode) -> String {
    Keybindings::keycode_to_string(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keybindings_defaults() {
        let defaults = Keybindings::defaults();
        assert_eq!(defaults.get("q"), Some(&Action::Quit));
        assert_eq!(defaults.get("j"), Some(&Action::MoveDown));
        assert_eq!(defaults.get("k"), Some(&Action::MoveUp));
        assert_eq!(defaults.get("h"), Some(&Action::MoveLeft));
        assert_eq!(defaults.get("l"), Some(&Action::MoveRight));
        assert_eq!(defaults.get("g"), Some(&Action::GoTop));
        assert_eq!(defaults.get("G"), Some(&Action::GoBottom));
        assert_eq!(defaults.get("~"), Some(&Action::GoHome));
        assert_eq!(defaults.get("."), Some(&Action::ToggleHidden));
    }

    #[test]
    fn test_keycode_to_string_char() {
        assert_eq!(keycode_to_string(&KeyCode::Char('a')), "a");
        assert_eq!(keycode_to_string(&KeyCode::Char('G')), "G");
    }

    #[test]
    fn test_keycode_to_string_special() {
        assert_eq!(keycode_to_string(&KeyCode::Up), "up");
        assert_eq!(keycode_to_string(&KeyCode::Down), "down");
        assert_eq!(keycode_to_string(&KeyCode::Left), "left");
        assert_eq!(keycode_to_string(&KeyCode::Right), "right");
        assert_eq!(keycode_to_string(&KeyCode::Enter), "enter");
        assert_eq!(keycode_to_string(&KeyCode::Esc), "esc");
        assert_eq!(keycode_to_string(&KeyCode::Backspace), "backspace");
    }

    #[test]
    fn test_keybindings_get_action_from_custom() {
        let mut bindings = Keybindings::default();
        bindings.bindings.insert("x".to_string(), Action::Quit);
        assert_eq!(bindings.get_action(&KeyCode::Char('x')), Some(Action::Quit));
    }

    #[test]
    fn test_keybindings_get_action_fallback() {
        let bindings = Keybindings::default();
        assert_eq!(bindings.get_action(&KeyCode::Char('q')), Some(Action::Quit));
    }

    #[test]
    fn test_keybindings_get_action_unknown() {
        let bindings = Keybindings::default();
        assert_eq!(bindings.get_action(&KeyCode::Char('z')), None);
    }

    #[test]
    fn test_action_serde_roundtrip() {
        let action = Action::MoveDown;
        let json = serde_json::to_string(&action).unwrap();
        let deserialized: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(action, deserialized);
    }
}
