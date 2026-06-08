# VHS-86

> *A retro terminal file manager with synthwave aesthetics.*

![synthwave](https://img.shields.io/badge/vibe-synthwave-magenta?style=flat-square)
![rust](https://img.shields.io/badge/language-rust-orange?style=flat-square)
![license](https://img.shields.io/badge/license-MIT-blue?style=flat-square)

VHS-86 is a terminal-native file manager built in Rust, designed for users who love vim keys, neon colors, and the aesthetic of an 80s CRT monitor. Navigate your filesystem like it's 1986 — but with modern performance.

---

## Features

- **Vim-style navigation** — `h` `j` `k` `l` (or arrow keys) to move around
- **Dual-pane layout** — file list on the left, live preview on the right
- **Synthwave color scheme** — deep violet blacks, hot magenta, electric cyan, neon green, and gold highlights
- **Directory preview** — see contents of folders without entering them
- **File preview** — text files rendered with syntax-aware truncation
- **Hidden file toggle** — press `.` to show/hide dotfiles
- **Jump to home** — press `~` to teleport to `$HOME`
- **Numeric jump** — type a number to jump directly to that file index
- **Fast & lightweight** — native Rust, minimal dependencies

---

## Install

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

---

## Usage

```bash
vhs-86 [path]
```

If no path is given, VHS-86 opens in the current directory.

### Keybindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `h` / `←` | Go to parent directory |
| `l` / `→` / `Enter` | Open selected directory |
| `g` | Jump to top |
| `G` | Jump to bottom |
| `~` | Go to home directory |
| `.` | Toggle hidden files |
| `0-9` | Jump to file by index |
| `q` | Quit |

---

## Roadmap

- [ ] File operations (copy, move, delete, rename)
- [ ] Fuzzy search / filter
- [ ] Bookmarks / quick-jump list
- [ ] Image preview via sixel/kitty graphics protocol
- [ ] Custom color theme support
- [ ] Config file (TOML)
- [ ] Shell integration (cd on quit)
- [ ] Trash support
- [ ] Bulk rename
- [ ] Git status indicators

---

## License

MIT © synthalorian

---

*Made with neon dreams and terminal love.*
