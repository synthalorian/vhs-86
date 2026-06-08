% VHS-86(1) vhs-86 0.7.0 | Terminal File Manager
% synthalorian
% June 2025

# NAME

vhs-86 - A retro terminal file manager with synthwave aesthetics

# SYNOPSIS

**vhs-86** [*OPTIONS*] [*PATH*]

# DESCRIPTION

VHS-86 is a terminal-native file manager built in Rust, designed for users who love vim keys, neon colors, and the aesthetic of an 80s CRT monitor. Navigate your filesystem like it's 1986 — but with modern performance.

# OPTIONS

**-h**, **--help**
:   Print help information

**-V**, **--version**
:   Print version information

**--no-config**
:   Run without loading the configuration file

**--theme** *THEME*
:   Override the default theme (default: synthwave)

**--show-hidden**
:   Show hidden files by default

**--no-preview**
:   Disable file preview pane

**--cd-on-quit**
:   Print current directory to stdout on quit (for shell integration)

# ARGUMENTS

[*PATH*]
:   The directory to open. Defaults to the current working directory.

# KEYBINDINGS

**j**, **Down**
:   Move down

**k**, **Up**
:   Move up

**h**, **Left**
:   Go to parent directory

**l**, **Right**, **Enter**
:   Open selected directory or file

**g**
:   Jump to top

**G**
:   Jump to bottom

**~**
:   Go to home directory

**.**
:   Toggle hidden files

**0-9**
:   Jump to file by index

**Space**
:   Toggle selection for batch operations

**/**
:   Open search dialog

**!**
:   Open shell command dialog

**c**
:   Open chmod dialog

**d**
:   Open disk usage view

**r**
:   Open remote connection dialog

**D**
:   Batch delete selected files

**C**
:   Batch copy selected files

**M**
:   Batch move selected files

**R**
:   Refresh directory listing

**q**
:   Quit

# CONFIGURATION

Configuration is stored at **$XDG_CONFIG_HOME/vhs-86/config.toml** (typically **~/.config/vhs-86/config.toml**).

Example configuration:

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

# SHELL INTEGRATION

Add the following to your shell configuration to enable "cd on quit":

**Bash/Zsh:**

```bash
v() {
    local dir
    dir=$(vhs-86 --cd-on-quit "$@")
    [ -n "$dir" ] && cd "$dir"
}
```

**Fish:**

```fish
function v
    set dir (vhs-86 --cd-on-quit $argv)
    test -n "$dir"; and cd "$dir"
end
```

# FILES

**~/.config/vhs-86/config.toml**
:   User configuration file

**~/.config/vhs-86/plugins/**
:   Plugin directory for WASM extensions

# ENVIRONMENT

**HOME**
:   Used to resolve the home directory for the **~** keybinding

**SHELL**
:   Default shell for executing commands

**XDG_CONFIG_HOME**
:   Base directory for configuration files

# EXAMPLES

Open current directory:

    vhs-86

Open specific directory:

    vhs-86 /path/to/directory

Open with hidden files shown:

    vhs-86 --show-hidden ~/documents

# BUGS

Report bugs at: <https://github.com/synthalorian/vhs-86/issues>

# SEE ALSO

**ls**(1), **cd**(1), **find**(1), **rg**(1)

# AUTHOR

Written by synthalorian.

# LICENSE

MIT License
