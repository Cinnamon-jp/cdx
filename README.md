# cdx

[![Crates.io](https://img.shields.io/crates/v/cdx-rs.svg)](https://crates.io/crates/cdx-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**cdx** (Accelerated cd) is a modern, simple and interactive `cd` command alternative for CLI lovers, written in Rust. It provides a fast and light TUI to incrementally search and navigate directories with ease.

## Features

- **Interactive TUI**: Navigate through directories with an intuitive terminal interface.
- **Non-Interactive Mode**: Supports standard `cd`-like usage with arguments for a seamless transition.
- **Incremental Search**: Quickly filter directories with minimal typing.
- **Case-Insensitive Search**: Search without distinguishing between uppercase and lowercase letters.
- **Keyboard-Driven**: Use standard navigation keys for fast and seamless movement.
- **Cross-Platform**: Built with `crossterm`, ensuring smooth execution on various platforms.

## Installation

Ensure you have Rust and Cargo installed. Then, install `cdx` via [crates.io](https://crates.io/crates/cdx-rs):

```bash
cargo install cdx-rs
```

<details>
<summary>Other Installation Methods</summary>

**From GitHub repository:**
```bash
cargo install --git https://github.com/Cinnamon-jp/cdx.git
```

**From local source:**
```bash
git clone https://github.com/Cinnamon-jp/cdx.git
cd cdx
cargo install --path .
```
</details>

### Uninstallation

To remove `cdx`, run:

```bash
cargo uninstall cdx-rs
```

## Shell Integration

Because `cdx` runs as a child process, it cannot directly change the working directory of your current shell. Instead, it prints the selected directory's absolute path to the standard output. 

To make it work as a seamless `cd` replacement, add the initialization command to your shell configuration file.

> **Note:** Ensure that the cargo installation directory (typically `~/.cargo/bin`) is included in your system's `$PATH`. Otherwise, the shell will not be able to find the `cdx` binary.

### bash
Add the following to your `~/.bashrc`:
```bash
eval "$(cdx init bash)"
```

### zsh
Add the following to your `~/.zshrc`:
```zsh
eval "$(cdx init zsh)"
```

### fish
Add the following to your `~/.config/fish/config.fish`:
```fish
cdx init fish | source
```

<details>
<summary>Manual Configuration (without eval)</summary>

If you prefer to define the wrapper function manually without `eval`:

**bash / zsh:**
```bash
function cdx() {
    if [ "$1" = "init" ]; then
        command cdx "$@"
        return
    fi
    local dest
    dest=$(command cdx "$@")
    if [ -n "$dest" ] && [ -d "$dest" ]; then
        builtin cd "$dest"
    fi
}
```

**fish:**
```fish
function cdx
    if test (count $argv) -gt 0 -a "$argv[1]" = "init"
        command cdx $argv
        return
    end
    set dest (command cdx $argv)
    if test -n "$dest" -a -d "$dest"
        builtin cd "$dest"
    end
end
```
</details>

## Usage

You can use `cdx` in two ways (very simple!!):

1. **Interactive Mode**: Simply type `cdx` without arguments to open the simple TUI.
2. **Direct Mode**: Type `cdx <directory>` to get the absolute path of the target directory. (Useful for scripting or quick resolution).

### Keybindings (TUI Mode)

| Key | Action |
| --- | --- |
| `Up` / `Down` | Move selection up or down. |
| `Tab` | Enter the selected directory (or go up if `..` is selected). |
| `Enter` | Confirm and change to the selected directory (or parent/current directory if `..` / `.` is selected). |
| `Backspace` | Delete the last typed character in the search. If search is empty, go up to the parent directory. |
| `Esc` / `Ctrl-C` | Cancel and exit without changing the directory. |

### TOML Configuration
You can create `config.toml` in `~/.config/cdx/` to configure **cdx**.

```toml
[ui]
selected_background_color = "<color>"
selected_foreground_color = "<color>"
path_foreground_color = "<color>"

[system]
case_insensitive_search = true|false
partial_navigation_fallback = true|false
```

- `<color>`:
  - Named colors: `black` | `gray` | `white` | `red` | `green` | `yellow` | `blue` | `magenta` | `cyan`  
    (Prefix with `dark ` for darker shades, e.g., `dark red`. `dark white` is an alias for `gray`.)
  - Hex RGB colors: `#RGB` or `#RRGGBB` (e.g., `#f00`, `#ff0000`)

> **Tip:** You can use [`config.schema.json`](config.schema.json) for validation and autocompletion in editors supporting JSON Schema (e.g. Even Better TOML).

## Planned Features

- **Colorize Entries List**: Use distinct colors for directories, symbolic links, hidden entries, and other entry types to make the list easier to scan.
- **Easy Installation**: Distribute pre-compiled binaries via package managers like Homebrew.
- **Hidden & Gitignore Support**: Add toggles for hidden directories and respect `.gitignore` rules.
- **Vim Keybindings**: Support `h`/`j`/`k`/`l` navigation for power users.
- **Directory Bookmarks**: Save and jump to your favorite directories instantly.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.