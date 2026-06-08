use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/// Parse a chmod-style numeric string (e.g., "755", "644")
pub fn parse_numeric_mode(mode_str: &str) -> Option<u32> {
    if mode_str.len() != 3 {
        return None;
    }
    let mut mode = 0u32;
    for (i, c) in mode_str.chars().enumerate() {
        let digit = c.to_digit(8)?;
        if digit > 7 {
            return None;
        }
        mode |= digit << ((2 - i) * 3);
    }
    Some(mode)
}

/// Convert a numeric mode to a human-readable string like "rwxr-xr-x"
pub fn mode_to_string(mode: u32) -> String {
    let perms = [
        (0o400, 'r'), (0o200, 'w'), (0o100, 'x'),
        (0o040, 'r'), (0o020, 'w'), (0o010, 'x'),
        (0o004, 'r'), (0o002, 'w'), (0o001, 'x'),
    ];
    perms
        .iter()
        .map(|(mask, ch)| if mode & *mask != 0 { *ch } else { '-' })
        .collect()
}

/// Convert a human-readable permission string to a numeric mode
pub fn string_to_mode(perm_str: &str) -> Option<u32> {
    if perm_str.len() != 9 {
        return None;
    }
    let mut mode = 0u32;
    let chars: Vec<char> = perm_str.chars().collect();
    let masks = [
        (0, 0o400), (1, 0o200), (2, 0o100),
        (3, 0o040), (4, 0o020), (5, 0o010),
        (6, 0o004), (7, 0o002), (8, 0o001),
    ];
    for (idx, mask) in masks {
        if chars[idx] != '-' {
            mode |= mask;
        }
    }
    Some(mode)
}

/// Get the current permissions of a file as a numeric mode
pub fn get_file_mode(path: &Path) -> Option<u32> {
    let meta = fs::metadata(path).ok()?;
    Some(meta.permissions().mode() & 0o777)
}

/// Set file permissions using a numeric mode string
pub fn set_file_mode(path: &Path, mode_str: &str) -> Result<(), String> {
    let mode = parse_numeric_mode(mode_str).ok_or_else(|| format!("Invalid mode: {}", mode_str))?;
    let meta = fs::metadata(path).map_err(|e| format!("Cannot read metadata: {}", e))?;
    let mut perms = meta.permissions();
    perms.set_mode(mode);
    fs::set_permissions(path, perms).map_err(|e| format!("Cannot set permissions: {}", e))?;
    Ok(())
}

/// Validate a permission input character (0-7)
pub fn is_valid_mode_char(c: char) -> bool {
    c.is_ascii_digit() && c >= '0' && c <= '7'
}

/// A dialog state for editing permissions
#[derive(Debug, Clone)]
pub struct ChmodDialog {
    pub visible: bool,
    pub path: Option<std::path::PathBuf>,
    pub input: String,
    pub current_mode: Option<u32>,
}

impl ChmodDialog {
    pub fn new() -> Self {
        Self {
            visible: false,
            path: None,
            input: String::new(),
            current_mode: None,
        }
    }

    pub fn open(&mut self, path: &Path) {
        self.visible = true;
        self.path = Some(path.to_path_buf());
        self.current_mode = get_file_mode(path);
        self.input = self.current_mode.map(|m| format!("{:03o}", m)).unwrap_or_default();
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.path = None;
        self.input.clear();
        self.current_mode = None;
    }

    pub fn push_char(&mut self, c: char) {
        if self.input.len() < 3 && is_valid_mode_char(c) {
            self.input.push(c);
        }
    }

    pub fn pop_char(&mut self) {
        self.input.pop();
    }

    pub fn apply(&self) -> Result<(), String> {
        if let Some(ref path) = self.path {
            set_file_mode(path, &self.input)
        } else {
            Err("No file selected".to_string())
        }
    }

    pub fn formatted_mode(&self) -> String {
        if let Some(mode) = self.current_mode {
            format!("{:03o} ({})", mode, mode_to_string(mode))
        } else {
            "???".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_numeric_mode_valid() {
        assert_eq!(parse_numeric_mode("755"), Some(0o755));
        assert_eq!(parse_numeric_mode("644"), Some(0o644));
        assert_eq!(parse_numeric_mode("777"), Some(0o777));
        assert_eq!(parse_numeric_mode("000"), Some(0o000));
    }

    #[test]
    fn test_parse_numeric_mode_invalid_length() {
        assert_eq!(parse_numeric_mode("75"), None);
        assert_eq!(parse_numeric_mode("7555"), None);
    }

    #[test]
    fn test_parse_numeric_mode_invalid_chars() {
        assert_eq!(parse_numeric_mode("78a"), None);
        assert_eq!(parse_numeric_mode("abc"), None);
    }

    #[test]
    fn test_parse_numeric_mode_out_of_range() {
        assert_eq!(parse_numeric_mode("788"), None);
    }

    #[test]
    fn test_mode_to_string() {
        assert_eq!(mode_to_string(0o755), "rwxr-xr-x");
        assert_eq!(mode_to_string(0o644), "rw-r--r--");
        assert_eq!(mode_to_string(0o777), "rwxrwxrwx");
        assert_eq!(mode_to_string(0o000), "---------");
    }

    #[test]
    fn test_string_to_mode_valid() {
        assert_eq!(string_to_mode("rwxr-xr-x"), Some(0o755));
        assert_eq!(string_to_mode("rw-r--r--"), Some(0o644));
    }

    #[test]
    fn test_string_to_mode_invalid_length() {
        assert_eq!(string_to_mode("rwxr-xr"), None);
    }

    #[test]
    fn test_is_valid_mode_char() {
        assert!(is_valid_mode_char('0'));
        assert!(is_valid_mode_char('7'));
        assert!(!is_valid_mode_char('8'));
        assert!(!is_valid_mode_char('a'));
    }

    #[test]
    fn test_chmod_dialog_new() {
        let dialog = ChmodDialog::new();
        assert!(!dialog.visible);
        assert!(dialog.path.is_none());
        assert!(dialog.current_mode.is_none());
    }

    #[test]
    fn test_chmod_dialog_push_char_valid() {
        let mut dialog = ChmodDialog::new();
        dialog.push_char('7');
        assert_eq!(dialog.input, "7");
        dialog.push_char('5');
        assert_eq!(dialog.input, "75");
        dialog.push_char('5');
        assert_eq!(dialog.input, "755");
    }

    #[test]
    fn test_chmod_dialog_push_char_invalid() {
        let mut dialog = ChmodDialog::new();
        dialog.push_char('8');
        assert_eq!(dialog.input, "");
    }

    #[test]
    fn test_chmod_dialog_push_char_max_length() {
        let mut dialog = ChmodDialog::new();
        dialog.push_char('7');
        dialog.push_char('5');
        dialog.push_char('5');
        dialog.push_char('5');
        assert_eq!(dialog.input, "755");
    }

    #[test]
    fn test_chmod_dialog_pop_char() {
        let mut dialog = ChmodDialog::new();
        dialog.push_char('7');
        dialog.push_char('5');
        dialog.pop_char();
        assert_eq!(dialog.input, "7");
        dialog.pop_char();
        assert_eq!(dialog.input, "");
        dialog.pop_char();
        assert_eq!(dialog.input, "");
    }
}
