# 📼 VHS-86

A retro terminal file manager with synthwave aesthetics. Navigate your filesystem like it's 1986.

## Features

- 🎹 **Synthwave color palette** — deep violet blacks, hot magenta, electric cyan
- 🖼️ **Image previews** — render actual image thumbnails in the preview pane (terminal permitting)
- ⌨️ **Vim-key navigation** — `h/j/k/l`, `gg`/`G`, `Ctrl+d`/`Ctrl+u`, numeric jumps
- 📂 **Dual-pane layout** — file list + live preview
- 🖱️ **Mouse support** — click to select, scroll to navigate
- 🗂️ **Smart sorting** — directories first, then by name, size, or modification time
- 🔍 **File filtering** — filter by extension on the fly
- 👻 **Hidden file toggle** — press `.` to show/hide dotfiles
- ✂️ **File operations** — copy, move, delete, rename with confirmation
- 📦 **Batch operations** — multi-select with Space, batch rename with sequential patterns
- 🔎 **Fuzzy find** — `/` to search files in the current directory
- ⚙️ **Config file** — customize colors and keybindings via `~/.config/vhs-86/config.toml`
- 🎨 **Built-in themes** — synthwave '84, midnight green, amber CRT

## Install

```bash
cargo build --release
sudo cp target/release/vhs-86 /usr/local/bin/
```

## Usage

```bash
vhs-86                    # Open in current directory
vhs-86 /path/to/dir       # Open specific directory
```

## Keybindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `h` / `←` | Parent directory |
| `l` / `→` / `Enter` | Open file/dir |
| `gg` | Go to top |
| `G` | Go to bottom |
| `Ctrl+d` | Half-page down |
| `Ctrl+u` | Half-page up |
| `~` | Go to home directory |
| `.` | Toggle hidden files |
| `/` | Fuzzy find files |
| `Space` | Toggle batch selection |
| `c` | Copy selected file(s) to clipboard |
| `m` | Cut (move) selected file(s) to clipboard |
| `p` | Paste clipboard into current directory |
| `d` | Delete selected file(s) (with confirmation) |
| `r` | Rename selected file/dir |
| `R` | Batch rename selected files with pattern |
| `y` | Confirm destructive action |
| `n` / `Esc` | Cancel prompt/operation |
| `q` / `Esc` | Quit |

### Search Mode

Press `/` to enter fuzzy find. Type to filter, `↑/↓` to navigate matches, `Enter` to jump to the selected file, `Esc` to cancel.

### Rename Mode

Press `r` to rename the selected item. Type the new name and press `Enter` to confirm, or `Esc` to cancel.

### Batch Rename

Select multiple files with `Space`, then press `R` to enter batch rename mode. Type a pattern and press `Enter`.

**Pattern syntax:**
- `vacation_{:03}.jpg` → `vacation_001.jpg`, `vacation_002.jpg`, ...
- `img_{}.png` → `img_1.png`, `img_2.png`, ...
- `backup` → `backup_1`, `backup_2`, ... (number appended)

### Copy / Move

Press `c` to copy or `m` to cut the selected item(s). Navigate to the target directory and press `p` to paste. If a file with the same name exists, you'll be prompted to overwrite.

### Batch Selection

Press `Space` to toggle selection on the current item. Selected items are highlighted in hot pink with a `▸` prefix. Then use `c`, `m`, or `d` to operate on all selected items at once.

## Command Palette

Press `:` to open the command palette. Type a command and press `Enter`.

| Command | Description |
|---------|-------------|
| `:quit` / `:q` | Exit |
| `:reload` / `:r` | Reload directory |
| `:hidden` / `:h` | Toggle hidden files |
| `:theme <name>` / `:t <name>` | Switch theme |
| `:clear` / `:cls` | Clear selection and clipboard |
| `:mkdir <name>` | Create a new directory |
| `:touch <name>` | Create an empty file |
| `:open` / `:o` | Open selected file with system default app |
| `:sort <name\|size\|modified>` / `:s <...>` | Sort files |
| `:filter <ext>` / `:f <ext>` | Filter files by extension |
| `:batchrename <pattern>` / `:br <pattern>` | Batch rename selected files |

## Themes

VHS-86 comes with three built-in themes:

- **synthwave-84** (default) — magenta highlights, cyan directories, black background
- **midnight-green** — teal highlights, mint directories, dark background
- **amber-crt** — amber highlights, golden directories, black background

Switch themes with `:theme <name>`. The active theme is persisted to `~/.config/vhs-86/config.toml`.

You can also create custom themes by adding `.toml` files to `~/.config/vhs-86/themes/`.

## Configuration

On first run, VHS-86 creates `~/.config/vhs-86/config.toml` with default values. Edit it to customize colors and keybindings.

### Example `config.toml`

```toml
[colors]
background = "black"
foreground = "white"
highlight_bg = "magenta"
highlight_fg = "black"
directory = "cyan"
file = "white"
border = "magenta"
status = "yellow"

# Active theme name. Built-in: synthwave-84, midnight-green, amber-crt
active_theme = "synthwave-84"

# Optional keybinding overrides.
# Use a single character or special name: enter, esc, backspace.
[keys]
# quit = "q"
# up = "k"
# down = "j"
# left = "h"
# right = "l"
# top = "g"
# bottom = "G"
# home = "~"
# toggle_hidden = "."
# copy = "c"
# move = "m"
# delete = "d"
# rename = "r"
# search = "/"
# confirm = "y"
# cancel = "n"
```

### Colors

Any of the following names are valid, plus `rgb(r,g,b)` for custom colors:

`black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `gray`, `dark_gray`, `light_red`, `light_green`, `light_yellow`, `light_blue`, `light_magenta`, `light_cyan`, `white`

## Image Previews

VHS-86 can render actual image thumbnails in the preview pane for supported terminals (Kitty, iTerm2, Foot, Xterm with Sixel, etc.). If your terminal doesn't support image protocols, it falls back to displaying image dimensions.

Supported image formats: PNG, JPG, JPEG, GIF, BMP, WebP, TIFF, ICO, AVIF.

## Roadmap

- [x] File operations (copy, move, delete, rename)
- [x] Better previews (image thumbnails + dimensions fallback)
- [x] Config file (`~/.config/vhs-86/config.toml`)
- [x] Custom themes (3 built-in + custom theme files)
- [x] Fuzzy find (`/search`)
- [x] Batch operations (multi-select, batch rename)
- [x] Vim navigation (`gg`, `G`, `Ctrl+d`, `Ctrl+u`)
- [x] Command palette expansion (`mkdir`, `touch`, `open`, `sort`, `filter`)
- [x] Theme persistence

### Phase 2.5 Ideas

- Syntax-highlighted text previews
- Configurable image preview sizing/fit modes
- Recursive/global search
- Custom user-defined commands in config
- Operation progress bars for large file transfers

## License

MIT
