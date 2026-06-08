# VHS-86

> *A retro terminal file manager with synthwave aesthetics.*

![Version](https://img.shields.io/badge/version-0.9.0--rc-magenta?style=flat-square)
![Rust](https://img.shields.io/badge/language-rust-orange?style=flat-square)
![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)
![Platform](https://img.shields.io/badge/platform-linux%20%7C%20macos%20%7C%20windows-cyan?style=flat-square)

VHS-86 is a terminal-native file manager built in Rust, designed for users who love vim keys, neon colors, and the aesthetic of an 80s CRT monitor. Navigate your filesystem like it's 1986 — but with modern performance and a comprehensive feature set.

![VHS-86 Screenshot](https://github.com/synthalorian/vhs-86/raw/main/assets/screenshot.png)

---

## Features

### Core Navigation
- **Vim-style navigation** — `h` `j` `k` `l` (or arrow keys) to move around
- **Dual-pane layout** — file list on the left, live preview on the right
- **Numeric jump** — type a number to jump directly to that file index
- **Directory stack** — navigate back through parent directories seamlessly

### File Operations
- **File preview** — text files rendered with syntax-aware highlighting via syntect
- **Directory preview** — see contents of folders without entering them
- **Archive browsing** — browse zip, tar, and tar.gz archives without extracting
- **Hidden file toggle** — press `.` to show/hide dotfiles
- **Jump to home** — press `~` to teleport to `$HOME`

### Advanced Operations (v0.5.0+)
- **Batch operations** — select multiple files with `Space`, then copy/move/delete
- **Permission editor** — press `c` to open chmod dialog with numeric mode input
- **Disk usage analyzer** — press `d` for treemap-style disk usage view
- **Remote filesystem** — press `r` to connect to SSH/SFTP servers

### Search & Integration (v0.6.0+)
- **Content search** — press `/` for ripgrep-powered file content search
- **Shell commands** — press `!` to execute shell commands in current directory
- **Git status indicators** — see added (`+`), modified (`M`), and untracked (`?`) files
- **Plugin system** — WASM-based extensions for custom functionality

### Customization
- **Synthwave color scheme** — deep violet blacks, hot magenta, electric cyan, neon green, and gold highlights
- **Custom themes** — define your own colors in TOML format
- **Custom keybindings** — remap any key to any action via config
- **Config file** — TOML configuration at `~/.config/vhs-86/config.toml`

---

## Install

### From crates.io (when published)

```bash
cargo install vhs-86
```

### From source

```bash
git clone https://github.com/synthalorian/vhs-86.git
cd vhs-86
cargo build --release
```

The binary will be at `./target/release/vhs-86`. Copy it to your `$PATH`:

```bash
cp target/release/vhs-86 ~/.local/bin/
```

### Prerequisites

- [Rust](https://rustup.rs/) (1.80+)
- A terminal with truecolor support (most modern terminals)
- System dependencies:
  - **Linux**: `libssl-dev`, `pkg-config`, `libgit2-dev`
  - **macOS**: `libgit2` (via Homebrew)
  - **Windows**: No additional dependencies

### Package Managers

#### Arch Linux (AUR)

```bash
yay -S vhs-86
# or
paru -S vhs-86
```

#### macOS (Homebrew)

```bash
brew tap synthalorian/vhs-86
brew install vhs-86
```

#### Windows (Chocolatey)

```bash
choco install vhs-86
```

---

## Usage

```bash
vhs-86 [OPTIONS] [PATH]
```

If no path is given, VHS-86 opens in the current directory.

### Command-Line Options

| Option | Description |
|--------|-------------|
| `[PATH]` | Starting directory (default: current directory) |
| `-H`, `--hidden` | Show hidden files by default |
| `-t`, `--theme <THEME>` | Use a specific color theme |
| `-c`, `--config <FILE>` | Use a custom config file |
| `--no-preview` | Disable file preview panel |
| `--no-highlight` | Disable syntax highlighting in previews |
| `--cd-on-quit` | Print current directory on quit (for shell integration) |
| `--no-mouse` | Disable mouse support |
| `--migrate-config` | Migrate config to the latest format |
| `--feedback <MESSAGE>` | Send feedback message |
| `-h`, `--help` | Print help |
| `-V`, `--version` | Print version |

### Examples

```bash
# Open current directory
vhs-86

# Open specific directory
vhs-86 ~/Documents

# Open with hidden files shown
vhs-86 --hidden ~/projects

# Use midnight theme
vhs-86 --theme midnight ~/code

# Shell integration mode
vhs-86 --cd-on-quit

# Custom config file
vhs-86 --config ~/.config/vhs-86/work.toml ~/work

# Disable preview for slow systems
vhs-86 --no-preview /mnt/network-drive
```

---

## Keybindings

### Navigation

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `h` / `←` | Go to parent directory |
| `l` / `→` / `Enter` | Open selected directory or archive |
| `g` | Jump to top |
| `G` | Jump to bottom |
| `~` | Go to home directory |
| `0-9` | Jump to file by index (type numbers sequentially) |

### File Operations

| Key | Action |
|-----|--------|
| `.` | Toggle hidden files |
| `Space` | Toggle batch selection |
| `D` | Delete selected file or batch delete |
| `C` | Batch copy selected files |
| `M` | Batch move selected files |
| `c` | Open chmod dialog (permission editor) |
| `R` | Refresh directory listing |

### Advanced Features

| Key | Action |
|-----|--------|
| `/` | Open search dialog (ripgrep integration) |
| `!` | Open shell command dialog |
| `d` | Open disk usage analyzer |
| `r` | Open remote SSH connection / disconnect |

### General

| Key | Action |
|-----|--------|
| `q` | Quit (or close current dialog) |
| `Esc` | Cancel current dialog / return to normal mode |

---

## Configuration

VHS-86 reads configuration from `$XDG_CONFIG_HOME/vhs-86/config.toml` (typically `~/.config/vhs-86/config.toml`).

### Example Configuration

```toml
# VHS-86 Configuration File
# Place at: ~/.config/vhs-86/config.toml

# Theme: "synthwave" (default) or custom theme name
theme = "synthwave"

# Show hidden files on startup
show_hidden = false

[preview]
# Enable syntax highlighting for code files
syntax_highlight = true
# Enable image preview via Kitty graphics protocol
image_preview = true
# Maximum lines to show in file preview
max_lines = 100

[shell]
# Print current directory on quit (for shell integration)
cd_on_quit = true
# Shell to use for executing commands
shell_command = "/bin/bash"

[plugins]
# Enable plugin system
enabled = true
# Auto-load plugins from ~/.config/vhs-86/plugins/
auto_load = true

[keybindings]
# Custom keybindings override defaults
# Format: key = "action_name"
q = "quit"
j = "move_down"
k = "move_up"
```

### Available Actions for Keybindings

- `quit` — Exit VHS-86
- `move_down`, `move_up`, `move_left`, `move_right` — Navigation
- `go_top`, `go_bottom` — Jump to top/bottom
- `go_home` — Go to home directory
- `toggle_hidden` — Toggle hidden files
- `toggle_select` — Toggle batch selection
- `enter` — Open selected item
- `refresh` — Refresh directory listing
- `open_chmod` — Open permission editor
- `open_disk_usage` — Open disk usage analyzer
- `open_remote_connect` — Open remote connection dialog
- `open_search` — Open search dialog
- `open_shell` — Open shell command dialog
- `batch_delete`, `batch_copy`, `batch_move` — Batch operations

---

## Shell Integration

Add this to your shell rc file to enable "cd on quit" functionality:

### Bash / Zsh

```bash
v() {
    local dir
    dir=$(vhs-86 --cd-on-quit "$@")
    [ -n "$dir" ] && cd "$dir"
}
```

### Fish

```fish
function v
    set dir (vhs-86 --cd-on-quit $argv)
    test -n "$dir"; and cd "$dir"
end
```

### Usage with shell integration

```bash
v ~/projects    # Opens vhs-86, cd's to selected directory on quit
```

---

## Themes

VHS-86 includes the default **synthwave** theme. Custom themes can be defined by creating TOML files in the themes directory.

### Built-in Theme Colors (Synthwave)

| Element | Color | Hex |
|---------|-------|-----|
| Background | Deep Violet | `#0A021A` |
| Panel Background | Dark Purple | `#12042A` |
| Magenta | Hot Magenta | `#FF00FF` |
| Cyan | Electric Cyan | `#00FFFF` |
| Pink | Neon Pink | `#FF1493` |
| Yellow | Gold | `#FFD700` |
| Green | Neon Green | `#39FF14` |
| Red | Neon Red | `#FF3250` |
| Border | Purple | `#B400B4` |
| Highlight | Dark Violet | `#280050` |

---

## File Preview

VHS-86 provides rich file previews in the right panel:

- **Text files** — Syntax-highlighted with line numbers (powered by syntect)
- **Directories** — List of contents with file counts
- **Archives** — List of archive contents without extraction
- **Images** — Preview via Kitty graphics protocol (when supported)
- **Binary files** — File metadata and size information

---

## Search

Press `/` to open the search dialog. VHS-86 integrates with ripgrep for fast content search:

- Type your query and press `Enter` to search
- Navigate results with `j`/`k` or arrow keys
- Press `l` or `→` to jump to the file containing the match
- Press `Esc` to close search

---

## Batch Operations

1. Navigate to files using `j`/`k`
2. Press `Space` to select/deselect files
3. Press `C` to copy, `M` to move, or `D` to delete selected files
4. Enter destination path when prompted
5. Press `Enter` to confirm or `Esc` to cancel

---

## Remote Filesystem

Connect to remote servers via SSH/SFTP:

1. Press `r` to open the remote connection dialog
2. Enter host in format `user@host` or just `host`
3. VHS-86 connects and displays remote directory listing
4. Navigate remote filesystem with normal keybindings
5. Press `r` again to disconnect

---

## Plugin System

VHS-86 supports WASM-based plugins for extensibility:

- Plugins are loaded from `~/.config/vhs-86/plugins/`
- Plugins can add custom preview handlers
- Enable/disable via configuration file

---

## Development

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=info cargo run
```

### Project Structure

```
vhs-86/
├── src/
│   ├── main.rs          # Application entry point and UI
│   ├── lib.rs           # Shared types and utilities
│   ├── config.rs        # Configuration management
│   ├── theme.rs         # Color theme system
│   ├── keybindings.rs   # Keybinding configuration
│   ├── preview.rs       # File preview engine
│   ├── git.rs           # Git status integration
│   ├── search.rs        # Content search with ripgrep
│   ├── batch.rs         # Batch file operations
│   ├── permissions.rs   # File permission editor
│   ├── disk_usage.rs    # Disk usage analyzer
│   ├── remote.rs        # SSH/SFTP remote filesystem
│   ├── archive.rs       # Archive browsing (zip, tar, gz)
│   ├── plugins.rs       # WASM plugin system
│   ├── crash_reporter.rs # Crash reporting
│   └── profiling.rs     # Performance profiling
├── man/                 # Man page source
├── scripts/             # Build scripts
├── tests/               # Integration tests
├── benches/             # Benchmarks
└── packaging/           # Distribution packaging
```

---

## Troubleshooting

### Garbled colors

Ensure your terminal supports truecolor. Test with:

```bash
curl -s https://raw.githubusercontent.com/JohnMorales/dotfiles/master/colors/24-bit-color.sh | bash
```

### Slow preview on large files

Disable syntax highlighting or reduce `max_lines` in config:

```toml
[preview]
syntax_highlight = false
max_lines = 50
```

### Git status not showing

Ensure the current directory is inside a git repository and `libgit2` is installed.

### Remote connection fails

Check SSH key authentication and ensure the remote host is accessible:

```bash
ssh user@host "echo test"
```

---

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Commit your changes: `git commit -m 'Add amazing feature'`
4. Push to the branch: `git push origin feature/amazing-feature`
5. Open a Pull Request

---

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history and release notes.

---

## Roadmap

- [x] v0.1.0 — Initial scaffold with vim navigation
- [x] v0.2.0 — File operations (copy, move, delete, rename)
- [x] v0.3.0 — Search & navigation enhancements
- [x] v0.4.0 — Shell integration & previews
- [x] v0.5.0 — Advanced features (archives, remote, disk usage)
- [x] v0.6.0 — Integration & plugins
- [x] v0.7.0 — Pre-release polish (tests, CI/CD, cross-platform)
- [x] v0.8.0 — Stability (error handling, logging, crash reporter)
- [ ] v0.9.0 — Release candidate (documentation, packaging)
- [ ] v1.0.0 — Stable release

---

## License

MIT © synthalorian

---

*Made with neon dreams and terminal love.*
