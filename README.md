# cdx

**cdx** (Accelerated cd) is a modern, simple and interactive `cd` command alternative for CLI lovers, written in Rust. It provides a fast and light TUI to incrementally search and navigate directories with ease.

## Features

- **Interactive TUI**: Navigate through directories with an intuitive terminal interface.
- **Non-Interactive Mode**: Supports standard `cd`-like usage with arguments for a seamless transition.
- **Incremental Search**: Quickly filter directories with minimal typing.
- **Keyboard-Driven**: Use standard navigation keys for fast and seamless movement.
- **Cross-Platform**: Built with `crossterm`, ensuring smooth execution on various platforms.

## Installation

Ensure you have Rust and Cargo installed. Then, you can build and install `cdx` from source:

```bash
git clone <your-repo-url>
cd cdx
cargo install --path .
```

### Uninstallation

To remove `cdx`, run:

```bash
cargo uninstall cdx
```

## Shell Integration

Because `cdx` runs as a child process, it cannot directly change the working directory of your current shell. Instead, it prints the selected directory's absolute path to the standard output. 

To make it work as a seamless `cd` replacement, add a wrapper function to your shell configuration file.

> **Note:** Ensure that the cargo installation directory (typically `~/.cargo/bin`) is included in your system's `$PATH`. Otherwise, the shell wrapper will not be able to find the `cdx` binary.

### Bash / Zsh
Add the following to your `~/.bashrc` or `~/.zshrc`:
```bash
function cdx() {
    local dest
    dest=$(command cdx "$@")
    if [ -n "$dest" ] && [ -d "$dest" ]; then
        cd "$dest"
    fi
}
```

### Fish
Add the following to your `~/.config/fish/config.fish`:
```fish
function cdx
    set dest (command cdx $argv)
    if test -n "$dest" -a -d "$dest"
        cd "$dest"
    end
end
```

## Usage

You can use `cdx` in two ways (very simple!!):

1. **Interactive Mode**: Simply type `cdx` without arguments to open the simple TUI.
2. **Direct Mode**: Type `cdx <directory>` to get the absolute path of the target directory. (Useful for scripting or quick resolution).

### Keybindings (TUI Mode)

| Key | Action |
| --- | --- |
| `Up` / `Down` | Move selection up or down. |
| `Tab` | Enter the selected directory (or go up if `..` is selected). |
| `Enter` | If `.` is selected, confirm and exit, changing to the current directory. Otherwise, enter the selected directory. |
| `Backspace` | Delete the last typed character in the search. If search is empty, go up to the parent directory. |
| `Esc` / `Ctrl-C` | Cancel and exit without changing the directory. |

## Planned Features

- **TOML Configuration**: Customize colors, keybindings, and behavior via a configuration file.
- **Easy Installation**: Distribute pre-compiled binaries via package managers like Homebrew.
- **Auto Shell Integration**: Provide a setup command to automatically configure shell wrappers.
- **Hidden & Gitignore Support**: Add toggles for hidden directories and respect `.gitignore` rules.
- **Vim Keybindings**: Support `h`/`j`/`k`/`l` navigation for power users.
- **Directory Bookmarks**: Save and jump to your favorite directories instantly.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.