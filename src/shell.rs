// Copyright (c) 2026 Cinnamon-jp
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}

// pub type ShellType = Shell;

impl Shell {
    pub fn as_str(&self) -> &'static str {
        match self {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
        }
    }
}

impl fmt::Display for Shell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Shell {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "bash" => Ok(Shell::Bash),
            "zsh" => Ok(Shell::Zsh),
            "fish" => Ok(Shell::Fish),
            _ => Err(format!(
                "Unsupported shell '{}'. Supported shells: bash, zsh, fish",
                s.trim()
            )),
        }
    }
}

/// 指定されたシェル用のラッパー関数初期化スクリプトを返す
pub fn get_init_script(shell: Shell) -> &'static str {
    match shell {
        Shell::Bash | Shell::Zsh => {
            r#"function cdx() {
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
"#
        }
        Shell::Fish => {
            r#"function cdx
    if test (count $argv) -gt 0 -a "$argv[1]" = "init"
        command cdx $argv
        return
    end
    set dest (command cdx $argv)
    if test -n "$dest" -a -d "$dest"
        builtin cd "$dest"
    end
end
"#
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_from_str_valid() {
        assert_eq!("bash".parse::<Shell>(), Ok(Shell::Bash));
        assert_eq!("BASH".parse::<Shell>(), Ok(Shell::Bash));
        assert_eq!(" bash ".parse::<Shell>(), Ok(Shell::Bash));
        assert_eq!("zsh".parse::<Shell>(), Ok(Shell::Zsh));
        assert_eq!("Zsh".parse::<Shell>(), Ok(Shell::Zsh));
        assert_eq!("fish".parse::<Shell>(), Ok(Shell::Fish));
        assert_eq!("FISH".parse::<Shell>(), Ok(Shell::Fish));
    }

    #[test]
    fn test_shell_from_str_invalid() {
        assert!("unknown".parse::<Shell>().is_err());
        assert!("sh".parse::<Shell>().is_err());
        assert!("".parse::<Shell>().is_err());
    }

    #[test]
    fn test_shell_display_and_as_str() {
        assert_eq!(Shell::Bash.as_str(), "bash");
        assert_eq!(Shell::Zsh.as_str(), "zsh");
        assert_eq!(Shell::Fish.as_str(), "fish");
        assert_eq!(Shell::Bash.to_string(), "bash");
        assert_eq!(Shell::Zsh.to_string(), "zsh");
        assert_eq!(Shell::Fish.to_string(), "fish");
    }

    #[test]
    fn test_get_init_script_bash_zsh() {
        let script_bash = get_init_script(Shell::Bash);
        assert!(script_bash.contains("function cdx()"));
        assert!(script_bash.contains("command cdx \"$@\""));
        assert!(script_bash.contains("builtin cd \"$dest\""));
        assert!(script_bash.contains("[ \"$1\" = \"init\" ]"));

        let script_zsh = get_init_script(Shell::Zsh);
        assert_eq!(script_bash, script_zsh);
        assert_eq!(get_init_script(Shell::Bash), script_bash);
    }

    #[test]
    fn test_get_init_script_fish() {
        let script_fish = get_init_script(Shell::Fish);
        assert!(script_fish.contains("function cdx"));
        assert!(script_fish.contains("command cdx $argv"));
        assert!(script_fish.contains("builtin cd \"$dest\""));
        assert!(script_fish.contains("\"$argv[1]\" = \"init\""));
        assert_eq!(get_init_script(Shell::Fish), script_fish);
    }
}
