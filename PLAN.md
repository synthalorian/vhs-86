# VHS-86 Roadmap

## v0.1.0 ✅ — Initial scaffold
- Basic dual-pane file manager with vim navigation
- Synthwave color scheme
- Directory and file preview

## v0.2.0 — File operations
- [ ] Copy, move, delete, rename files
- [ ] Trash support (move to ~/.local/share/Trash)
- [ ] Bulk rename with regex patterns
- [ ] File creation (new file, new directory)

## v0.3.0 — Search & navigation
- [ ] Fuzzy search / filter with fzf-like scoring
- [ ] Bookmarks / quick-jump list (save favorite paths)
- [ ] Jump to previous directory stack (cd - history)
- [ ] Numeric jump improvements

## v0.4.0 — Phase 4: Shell integration & previews
- [ ] Shell integration — cd on quit (output path to stdout)
- [ ] Image preview via kitty graphics protocol
- [ ] Syntax-highlighted file preview (basic extension detection)
- [ ] Config file support (TOML at ~/.config/vhs-86/config.toml)
- [ ] Custom color theme support
- [ ] Git status indicators in file list

## v0.5.0 — Advanced features
- [ ] Archive support (zip, tar, gz) — browse without extracting
- [ ] Remote filesystem support (SSH/SFTP via russh)
- [ ] File permissions editor (chmod-style)
- [ ] Disk usage analyzer (tree-map view)
- [ ] Batch operations (select multiple, apply action)

## v0.6.0 — Integration & plugins
- [ ] Plugin system (WASM-based extensions)
- [ ] Integration with ripgrep for content search
- [ ] FZF-like preview window for search results
- [ ] Custom keybindings via config
- [ ] Shell command execution (!command)

## v0.7.0 — Pre-release polish
- [ ] Comprehensive test suite (unit + integration)
- [ ] CI/CD with GitHub Actions
- [ ] Cross-platform builds (Linux, macOS, Windows)
- [ ] Man page and --help overhaul
- [ ] Performance profiling and optimization

## v0.8.0 — Stability
- [ ] Error handling audit (no unwraps in production)
- [ ] Logging with tracing
- [ ] Crash reporter
- [ ] Config migration system
- [ ] Beta testing feedback integration

## v0.9.0 — Release candidate
- [ ] Final API freeze
- [ ] Documentation complete
- [ ] Packaging (cargo, AUR, Homebrew, Chocolatey)
- [ ] Security audit
- [ ] Release notes draft

## v1.0.0 — Ship it
- [ ] Tag v1.0.0
- [ ] Publish to crates.io
- [ ] Announcement post
- [ ] Screencast demo
- [ ] Community feedback channel
