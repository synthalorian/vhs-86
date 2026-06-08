# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.0] - 2025-06-08

### Release Candidate

This is the v0.9.0 release candidate, marking the final stage before the v1.0.0 stable release. All features are frozen and the API is stable.

### Added

- **Comprehensive Documentation**
  - Complete README with usage examples, configuration guide, and troubleshooting
  - Full man page with all keybindings, modes, and configuration options
  - Inline help via `--help` flag with examples and keybinding reference

- **Packaging Support**
  - Cargo publish validation script (`scripts/cargo-publish-check.sh`)
  - AUR PKGBUILD template for Arch Linux users
  - Homebrew formula template for macOS users
  - Chocolatey nuspec template for Windows users
  - Cross-platform build scripts for Linux, macOS, and Windows

- **Release Infrastructure**
  - CHANGELOG.md with structured release notes
  - Version consistency checks across Cargo.toml, README, and man page
  - Automated CI/CD pipeline with GitHub Actions
  - Cross-platform release binary builds

### Changed

- Updated version to 0.9.0 across all documentation and metadata
- Refined CLI argument descriptions for clarity
- Enhanced `--help` output with keybinding reference and examples

### Fixed

- Man page version now matches release version
- README version badge reflects current release

### Known Issues

- Remote filesystem support requires SSH key authentication (password auth not yet supported)
- Plugin system is experimental and WASM API may change before v1.0.0
- Image preview requires Kitty terminal or compatible emulator

## [0.8.0] - 2025-05-15

### Stability & Reliability

### Added

- **Crash Reporter**
  - Automatic crash detection and log collection
  - Crash logs stored at `~/.local/share/vhs-86/crash.log`
  - `--feedback` flag for submitting crash reports with user feedback

- **Logging**
  - Integrated `tracing` for structured logging
  - Configurable via `RUST_LOG` environment variable
  - Logs written to stderr for easy redirection

- **Config Migration**
  - `--migrate-config` flag to upgrade config files to latest format
  - Automatic migration detection on startup
  - Version tracking in config file to prevent data loss

### Changed

- Replaced all production `unwrap()` calls with proper error handling
- Improved error messages throughout the application
- Enhanced stability in file operations and remote connections

### Fixed

- Fixed potential panic when navigating to inaccessible directories
- Fixed memory leak in plugin system during repeated reloads
- Fixed race condition in git status cache refresh

## [0.7.0] - 2025-04-20

### Pre-release Polish

### Added

- **Comprehensive Test Suite**
  - Unit tests for core utilities (format_size, format_time, config, theme)
  - Integration tests for file operations
  - Documentation tests for public APIs
  - Benchmark suite with criterion

- **CI/CD Pipeline**
  - GitHub Actions workflow for Linux, macOS, and Windows
  - Automated formatting checks with `cargo fmt`
  - Clippy linting with warnings-as-errors
  - Code coverage reporting with cargo-tarpaulin

- **Cross-platform Builds**
  - Linux x86_64 binary builds
  - macOS x86_64 binary builds
  - Windows x86_64 binary builds
  - Automated artifact uploads

- **Man Page Generation**
  - `--generate-man-page` flag for generating man pages from clap definitions
  - Markdown man page source for easy editing

- **Performance Profiling**
  - Benchmark suite in `benches/`
  - Release profile optimization (LTO, strip)

### Changed

- Optimized release builds with `lto = true` and `strip = true`
- Improved rendering performance for large directories
- Reduced memory allocations during directory listing

### Fixed

- Fixed rendering glitches on Windows terminals
- Fixed mouse support detection on various terminal emulators
- Fixed git status indicator alignment for long filenames

## [0.6.0] - 2025-03-10

### Integration & Plugins

### Added

- **Plugin System**
  - WASM-based plugin architecture using Extism
  - Plugin directory at `~/.config/vhs-86/plugins/`
  - Auto-load plugins on startup (configurable)
  - Plugin preview hooks for custom file type handling

- **Content Search**
  - ripgrep integration for fast file content search
  - Search dialog with real-time results
  - Preview pane showing context around matches
  - Jump to file from search results

- **Shell Commands**
  - `!` key to open shell command dialog
  - Execute arbitrary commands in current directory
  - Full-screen output display
  - Error output captured and displayed

- **Custom Keybindings**
  - Configurable keybindings via TOML
  - All actions can be remapped
  - Support for vim-style and custom layouts

### Changed

- Refactored input handling to support modal dialogs
- Improved status bar with mode indicators

## [0.5.0] - 2025-02-01

### Advanced Features

### Added

- **Archive Support**
  - Browse zip archives without extracting
  - Browse tar and tar.gz archives
  - Archive contents shown in file list and preview
  - Enter archives like directories

- **Remote Filesystem**
  - SSH/SFTP connection support
  - Navigate remote directories with local keybindings
  - Remote file listing and preview
  - Disconnect to return to local filesystem

- **Permission Editor**
  - `c` key opens chmod dialog
  - Numeric mode input (e.g., 755, 644)
  - Current permissions display
  - Works on selected files

- **Disk Usage Analyzer**
  - `d` key opens disk usage view
  - Treemap-style visualization with bars
  - Sorted by size
  - Percentage display

- **Batch Operations**
  - `Space` to select multiple files
  - `C` to copy selected files
  - `M` to move selected files
  - `D` to delete selected files
  - Visual selection indicators

### Changed

- Redesigned status bar to show batch selection count
- Improved file type detection with archive support

## [0.4.0] - 2024-12-15

### Shell Integration & Previews

### Added

- **Shell Integration**
  - `--cd-on-quit` flag prints current directory on exit
  - Easy shell function setup for bash, zsh, and fish

- **Image Preview**
  - Kitty graphics protocol support
  - Automatic image file detection
  - Configurable via `image_preview` option

- **Syntax Highlighting**
  - Text file preview with syntax highlighting
  - Powered by syntect
  - Language detection from file extensions
  - Configurable via `syntax_highlight` option

- **Configuration File**
  - TOML config at `~/.config/vhs-86/config.toml`
  - Theme selection
  - Preview settings
  - Shell integration settings

- **Custom Themes**
  - Theme system with color definitions
  - Default synthwave theme
  - Theme loading from config

- **Git Status Indicators**
  - `+` for added files (green)
  - `M` for modified files (yellow)
  - `?` for untracked files (red)
  - Git repository detection

### Changed

- Major UI refresh with improved spacing and borders
- File list now shows modification times

## [0.3.0] - 2024-11-01

### Search & Navigation

### Added

- **Fuzzy Search Filter**
  - Real-time filtering as you type
  - fzf-like scoring algorithm
  - Case-insensitive matching

- **Bookmarks / Quick-jump**
  - Save favorite directories
  - Quick navigation to bookmarked paths

- **Directory Stack**
  - Navigate back through previously visited directories
  - Maintains navigation history

- **Numeric Jump Improvements**
  - Faster response time
  - Better handling of large directories
  - Visual feedback during number input

## [0.2.0] - 2024-09-20

### File Operations

### Added

- **File Operations**
  - Copy files and directories
  - Move/rename files and directories
  - Delete files and directories
  - Rename with regex patterns (bulk rename)

- **Trash Support**
  - Move to `~/.local/share/Trash` instead of permanent deletion
  - Trash directory creation
  - Safe deletion with undo capability

- **File Creation**
  - Create new files
  - Create new directories

## [0.1.0] - 2024-08-01

### Initial Release

### Added

- Basic dual-pane file manager
- Vim-style navigation (h, j, k, l)
- Synthwave color scheme
- Directory and file preview
- Hidden file toggle
- Home directory jump
- Basic file listing with sizes

---

[Unreleased]: https://github.com/synthalorian/vhs-86/compare/v0.9.0...HEAD
[0.9.0]: https://github.com/synthalorian/vhs-86/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/synthalorian/vhs-86/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/synthalorian/vhs-86/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/synthalorian/vhs-86/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/synthalorian/vhs-86/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/synthalorian/vhs-86/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/synthalorian/vhs-86/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/synthalorian/vhs-86/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/synthalorian/vhs-86/releases/tag/v0.1.0
