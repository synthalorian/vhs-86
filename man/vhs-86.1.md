% VHS-86(1) vhs-86 0.9.0 | Terminal File Manager
% synthalorian
% June 2025

# NAME

vhs-86 - A retro terminal file manager with synthwave aesthetics

# SYNOPSIS

**vhs-86** [*OPTIONS*] [*PATH*]

# DESCRIPTION

VHS-86 is a terminal-native file manager built in Rust, designed for users who love vim keys, neon colors, and the aesthetic of an 80s CRT monitor. Navigate your filesystem like it's 1986 — but with modern performance.

VHS-86 features a dual-pane layout with a file list on the left and a live preview on the right. It supports vim-style navigation, batch file operations, syntax-highlighted previews, archive browsing, remote filesystem access via SSH/SFTP, content search via ripgrep integration, git status indicators, and a WASM-based plugin system.

# OPTIONS

**-h**, **--help**
:   Print help information

**-V**, **--version**
:   Print version information

**-H**, **--hidden**
:   Show hidden files by default

**-t**, **--theme** *THEME*
:   Use a specific color theme (default: synthwave)

**-c**, **--config** *FILE*
:   Use a custom configuration file

**--no-preview**
:   Disable the file preview pane

**--no-highlight**
:   Disable syntax highlighting in file previews

**--cd-on-quit**
:   Print the current directory to stdout on quit (for shell integration)

**--no-mouse**
:   Disable mouse support

**--migrate-config**
:   Migrate the configuration file to the latest format and exit

**--feedback** *MESSAGE*
:   Send a feedback message and exit

# ARGUMENTS

[*PATH*]
:   The directory to open. Defaults to the current working directory.

# KEYBINDINGS

## Navigation

**j**, **Down**
:   Move down one item

**k**, **Up**
:   Move up one item

**h**, **Left**
:   Go to parent directory

**l**, **Right**, **Enter**
:   Open selected directory, archive, or file

**g**
:   Jump to top of file list

**G**
:   Jump to bottom of file list

**~**
:   Go to home directory

**0-9**
:   Jump to file by index. Type numbers sequentially within 800ms to navigate to specific indices.

## File Operations

**.**
:   Toggle hidden files visibility

**Space**
:   Toggle selection for batch operations

**D**
:   Delete selected file, or batch delete if items are selected

**C**
:   Batch copy selected files to destination

**M**
:   Batch move selected files to destination

**c**
:   Open chmod dialog to edit file permissions

**R**
:   Refresh directory listing

## Advanced Features

**/**
:   Open search dialog with ripgrep integration

**!**
:   Open shell command dialog

**d**
:   Open disk usage analyzer

**r**
:   Open remote SSH connection dialog, or disconnect if already connected

## General

**q**
:   Quit the application or close the current dialog

**Esc**
:   Cancel current dialog and return to normal mode

# MODES

VHS-86 operates in several modes, indicated by the current UI state:

**Normal Mode**
:   Default mode for navigation and file operations. Status bar shows current path and item count.

**Search Mode**
:   Activated by pressing **/**. Shows search query input and results with preview. Navigate results with **j**/**k** or arrow keys. Press **Enter** to execute search, **l** or **→** to jump to selected result.

**Shell Command Mode**
:   Activated by pressing **!**. Type shell commands to execute in the current directory. Output is displayed in a full-screen panel. Press any key to dismiss output.

**Chmod Mode**
:   Activated by pressing **c** on a selected file. Enter numeric permissions (e.g., 755, 644) and press **Enter** to apply.

**Batch Mode**
:   Activated when performing batch copy, move, or delete operations. Enter destination path and press **Enter** to confirm.

**Remote Connect Mode**
:   Activated by pressing **r**. Enter SSH target as **user@host** or just **host** to connect to a remote filesystem.

# CONFIGURATION

Configuration is stored at **$XDG_CONFIG_HOME/vhs-86/config.toml** (typically **~/.config/vhs-86/config.toml**).

## Configuration File Format

```toml
theme = "synthwave"
show_hidden = false

[preview]
syntax_highlight = true
image_preview = true
max_lines = 100

[shell]
cd_on_quit = true
shell_command = "/bin/bash"

[plugins]
enabled = true
auto_load = true

[keybindings]
q = "quit"
j = "move_down"
k = "move_up"
```

## Configuration Options

**theme**
:   Color theme name. Currently supported: "synthwave". Default: "synthwave"

**show_hidden**
:   Show hidden files on startup. Default: false

**preview.syntax_highlight**
:   Enable syntax highlighting for text file previews. Default: true

**preview.image_preview**
:   Enable image preview via Kitty graphics protocol. Default: true

**preview.max_lines**
:   Maximum lines to display in file preview. Default: 100

**shell.cd_on_quit**
:   Print current directory on quit when using --cd-on-quit flag. Default: true

**shell.shell_command**
:   Shell executable for running commands. Default: value of $SHELL or "/bin/sh"

**plugins.enabled**
:   Enable the plugin system. Default: true

**plugins.auto_load**
:   Automatically load plugins from plugin directory. Default: true

**keybindings**
:   Map keys to actions. See ACTIONS section for available action names.

# ACTIONS

The following action names can be used in the **[keybindings]** section of the configuration file:

- **quit** — Exit VHS-86
- **move_down** — Move selection down
- **move_up** — Move selection up
- **move_left** — Go to parent directory
- **move_right** — Open selected item
- **enter** — Open selected item (same as move_right)
- **go_top** — Jump to first item
- **go_bottom** — Jump to last item
- **go_home** — Navigate to home directory
- **toggle_hidden** — Toggle hidden files display
- **toggle_select** — Toggle batch selection for current item
- **refresh** — Refresh directory listing
- **open_chmod** — Open permission editor dialog
- **open_disk_usage** — Open disk usage analyzer
- **open_remote_connect** — Open remote connection dialog
- **open_search** — Open search dialog
- **open_shell** — Open shell command dialog
- **batch_delete** — Delete selected or batch-selected files
- **batch_copy** — Copy batch-selected files
- **batch_move** — Move batch-selected files

# SHELL INTEGRATION

Add the following to your shell configuration to enable "cd on quit":

## Bash / Zsh

```bash
v() {
    local dir
    dir=$(vhs-86 --cd-on-quit "$@")
    [ -n "$dir" ] && cd "$dir"
}
```

## Fish

```fish
function v
    set dir (vhs-86 --cd-on-quit $argv)
    test -n "$dir"; and cd "$dir"
end
```

# FILE PREVIEW

VHS-86 provides rich file previews in the right panel:

**Text Files**
:   Syntax-highlighted content powered by syntect, with configurable line limits.

**Directories**
:   List of directory contents with item counts.

**Archives**
:   Contents listing for zip, tar, and tar.gz files without extraction.

**Images**
:   Preview via Kitty graphics protocol when running in a compatible terminal.

**Binary Files**
:   File metadata, size, and permissions information.

# GIT INTEGRATION

When inside a git repository, VHS-86 displays status indicators:

**+**
:   Added files (green)

**M**
:   Modified files (yellow)

**?**
:   Untracked files (red)

**  **
:   Unchanged files (no indicator)

# SEARCH

The search dialog (**/**) provides content search using ripgrep integration:

- Type query and press **Enter** to search
- Results show file name, line number, and matching line
- Right panel shows context around the match
- Navigate with **j**/**k** or arrow keys
- Press **l** or **→** to jump to the file
- Press **Esc** to close

# BATCH OPERATIONS

Perform operations on multiple files:

1. Navigate to files and press **Space** to select/deselect
2. Selected items are indicated with underline
3. Press **C** to copy, **M** to move, or **D** to delete
4. Enter destination path (for copy/move) and press **Enter**
5. Press **Esc** to cancel

# REMOTE FILESYSTEM

Connect to remote servers via SSH/SFTP:

1. Press **r** to open connection dialog
2. Enter **user@host** or just **host**
3. Navigate remote filesystem with normal keybindings
4. Press **r** again to disconnect and return to local filesystem

# PLUGIN SYSTEM

VHS-86 supports WASM-based plugins for extensibility:

- Plugins are loaded from **~/.config/vhs-86/plugins/**
- Plugins can provide custom preview handlers
- Enable/disable via the **[plugins]** configuration section

# FILES

**~/.config/vhs-86/config.toml**
:   User configuration file

**~/.config/vhs-86/plugins/**
:   Plugin directory for WASM extensions

**~/.local/share/vhs-86/crash.log**
:   Crash reporter log file

# ENVIRONMENT

**HOME**
:   Used to resolve the home directory for the **~** keybinding

**SHELL**
:   Default shell for executing commands via the shell dialog

**XDG_CONFIG_HOME**
:   Base directory for configuration files (default: ~/.config)

**RUST_LOG**
:   Logging level for tracing (e.g., "info", "debug", "warn")

# EXAMPLES

Open current directory:

    vhs-86

Open specific directory:

    vhs-86 /path/to/directory

Open with hidden files shown:

    vhs-86 --hidden ~/documents

Use a custom theme:

    vhs-86 --theme midnight ~/code

Shell integration mode:

    vhs-86 --cd-on-quit

Use custom config file:

    vhs-86 --config ~/.config/vhs-86/work.toml ~/work

Disable preview for network drives:

    vhs-86 --no-preview /mnt/network-drive

Migrate configuration to latest format:

    vhs-86 --migrate-config

Send feedback:

    vhs-86 --feedback "Great file manager!"

# EXIT STATUS

**0**
:   Success

**1**
:   General error

**130**
:   Interrupted (Ctrl-C)

# BUGS

Report bugs at: <https://github.com/synthalorian/vhs-86/issues>

# SEE ALSO

**ls**(1), **cd**(1), **find**(1), **rg**(1), **chmod**(1), **ssh**(1), **bash**(1)

# AUTHOR

Written by synthalorian.

# LICENSE

MIT License
