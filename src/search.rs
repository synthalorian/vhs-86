use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A single ripgrep search result
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub column: usize,
    pub matched_text: String,
    pub line_text: String,
}

/// Run ripgrep and return parsed results
pub fn search(query: &str, path: &Path) -> io::Result<Vec<SearchResult>> {
    let output = Command::new("rg")
        .args([
            "--json",
            "--line-number",
            "--column",
            "--smart-case",
            "--max-count", "100",
            query,
        ])
        .current_dir(path)
        .output()?;

    let mut results = Vec::new();

    if !output.status.success() && output.status.code() != Some(1) {
        // rg exits with 1 when no matches found, which is ok
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("ripgrep error: {}", stderr),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(match_obj) = json.get("data").and_then(|d| d.get("submatches")).and_then(|s| s.as_array()) {
                for submatch in match_obj {
                    let file_path = json["data"]["path"]["text"]
                        .as_str()
                        .map(PathBuf::from)
                        .unwrap_or_else(|| path.to_path_buf());

                    let line_number = json["data"]["line_number"]
                        .as_u64()
                        .unwrap_or(1) as usize;

                    let line_text = json["data"]["lines"]["text"]
                        .as_str()
                        .unwrap_or("")
                        .to_string();

                    let matched_text = submatch["match"]["text"]
                        .as_str()
                        .unwrap_or("")
                        .to_string();

                    let column = submatch["start"]
                        .as_u64()
                        .unwrap_or(0) as usize;

                    results.push(SearchResult {
                        file_path,
                        line_number,
                        column,
                        matched_text,
                        line_text,
                    });
                }
            }
        }
    }

    Ok(results)
}

/// Search state for the UI
#[derive(Debug, Clone)]
pub struct SearchState {
    pub visible: bool,
    pub query: String,
    pub results: Vec<SearchResult>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub error: Option<String>,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            visible: false,
            query: String::new(),
            results: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            error: None,
        }
    }

    pub fn open(&mut self) {
        self.visible = true;
        self.query.clear();
        self.results.clear();
        self.selected = 0;
        self.scroll_offset = 0;
        self.error = None;
    }

    pub fn close(&mut self) {
        self.visible = false;
    }

    pub fn push_char(&mut self, c: char) {
        self.query.push(c);
    }

    pub fn pop_char(&mut self) {
        self.query.pop();
    }

    pub fn move_down(&mut self, n: usize) {
        if self.results.is_empty() { return; }
        self.selected = (self.selected + n).min(self.results.len() - 1);
    }

    pub fn move_up(&mut self, n: usize) {
        self.selected = self.selected.saturating_sub(n);
    }

    pub fn execute_search(&mut self, cwd: &Path) {
        if self.query.is_empty() {
            self.results.clear();
            return;
        }

        match search(&self.query, cwd) {
            Ok(results) => {
                self.results = results;
                self.selected = 0;
                self.scroll_offset = 0;
                self.error = None;
            }
            Err(e) => {
                self.error = Some(e.to_string());
                self.results.clear();
            }
        }
    }

    pub fn selected_result(&self) -> Option<&SearchResult> {
        self.results.get(self.selected)
    }
}

/// Get lines of context around a search result for preview
pub fn get_preview_lines(path: &Path, target_line: usize, context_lines: usize) -> Vec<(String, bool)> {
    let mut lines = Vec::new();

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => {
            lines.push(("[binary or unreadable file]".to_string(), false));
            return lines;
        }
    };

    let all_lines: Vec<&str> = content.lines().collect();
    let start = target_line.saturating_sub(context_lines + 1);
    let end = (target_line + context_lines).min(all_lines.len());

    for (idx, line) in all_lines.iter().enumerate().take(end).skip(start) {
        let line_num = idx + 1;
        let is_target = line_num == target_line;
        let formatted = format!("{:4} | {}", line_num, line);
        lines.push((formatted, is_target));
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_state_new() {
        let state = SearchState::new();
        assert!(!state.visible);
        assert!(state.query.is_empty());
        assert!(state.results.is_empty());
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn test_search_state_open() {
        let mut state = SearchState::new();
        state.open();
        assert!(state.visible);
        assert!(state.query.is_empty());
    }

    #[test]
    fn test_search_state_close() {
        let mut state = SearchState::new();
        state.open();
        state.close();
        assert!(!state.visible);
    }

    #[test]
    fn test_search_state_push_char() {
        let mut state = SearchState::new();
        state.push_char('a');
        state.push_char('b');
        assert_eq!(state.query, "ab");
    }

    #[test]
    fn test_search_state_pop_char() {
        let mut state = SearchState::new();
        state.push_char('a');
        state.pop_char();
        assert!(state.query.is_empty());
        state.pop_char();
        assert!(state.query.is_empty());
    }

    #[test]
    fn test_search_state_move_down() {
        let mut state = SearchState::new();
        state.results = vec![
            SearchResult {
                file_path: PathBuf::from("a.txt"),
                line_number: 1,
                column: 0,
                matched_text: "a".to_string(),
                line_text: "aaa".to_string(),
            },
            SearchResult {
                file_path: PathBuf::from("b.txt"),
                line_number: 2,
                column: 0,
                matched_text: "b".to_string(),
                line_text: "bbb".to_string(),
            },
        ];
        state.move_down(1);
        assert_eq!(state.selected, 1);
        state.move_down(1);
        assert_eq!(state.selected, 1);
    }

    #[test]
    fn test_search_state_move_up() {
        let mut state = SearchState::new();
        state.results = vec![
            SearchResult {
                file_path: PathBuf::from("a.txt"),
                line_number: 1,
                column: 0,
                matched_text: "a".to_string(),
                line_text: "aaa".to_string(),
            },
        ];
        state.selected = 1;
        state.move_up(1);
        assert_eq!(state.selected, 0);
        state.move_up(1);
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn test_search_state_selected_result() {
        let mut state = SearchState::new();
        assert!(state.selected_result().is_none());

        let result = SearchResult {
            file_path: PathBuf::from("a.txt"),
            line_number: 1,
            column: 0,
            matched_text: "a".to_string(),
            line_text: "aaa".to_string(),
        };
        state.results.push(result.clone());
        assert_eq!(state.selected_result().unwrap().line_text, "aaa");
    }

    #[test]
    fn test_get_preview_lines() {
        let tmpdir = tempfile::tempdir().unwrap();
        let file_path = tmpdir.path().join("test.txt");
        std::fs::write(&file_path, "line1\nline2\nline3\nline4\nline5\n").unwrap();

        let lines = get_preview_lines(&file_path, 3, 1);
        assert!(!lines.is_empty());
        let target = lines.iter().find(|(_, is_target)| *is_target);
        assert!(target.is_some());
        assert!(target.unwrap().0.contains("3"));
    }
}
