// Copyright (c) 2026 Cinnamon-jp
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

mod app;
mod config;
mod fs;
mod shell;
mod tui;
mod ui;

use std::{env, path, str::FromStr};

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = config::load_config_or_default();

    // Mode decision by args pattern
    let target_dir = match &args[1..] {
        // cdx init <shell>
        [cmd, shell] if cmd == "init" => {
            handle_init(shell);
        }
        // Invalid init args
        [cmd, ..] if cmd == "init" => {
            eprintln!("Usage: cdx init <bash|zsh|fish>");
            std::process::exit(1);
        }
        // Non args -> TUI mode
        [] => run_tui(get_current_dir(), config),
        // One arg -> specific path (Direct or Fallback)
        [path_arg] => resolve_target_dir(path_arg, config).unwrap_or_else(|err| {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }),
        // Too much args
        _ => {
            eprintln!("Usage: cdx [directory] | cdx init <shell>");
            std::process::exit(1);
        }
    };

    // Convert to absolute path
    match target_dir.canonicalize() {
        Ok(abs_path) => println!("{}", abs_path.display()),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ResolveError {
    NotFound(path::PathBuf),
    NotADirectory(path::PathBuf),
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolveError::NotFound(p) => write!(f, "'{}' does not exist.", p.display()),
            ResolveError::NotADirectory(p) => write!(f, "'{}' is not a directory.", p.display()),
        }
    }
}

impl std::error::Error for ResolveError {}

/// パス引数を解決 (有効なディレクトリなら直接採用、無効ならフォールバックTUIを起動)
fn resolve_target_dir(arg: &str, config: config::Config) -> Result<path::PathBuf, ResolveError> {
    let arg_path = path::PathBuf::from(arg);

    if arg_path.is_dir() {
        return Ok(arg_path);
    }

    if config.system.partial_navigation_fallback {
        let start_dir = find_deepest_dir(&arg_path).unwrap_or_else(get_current_dir);
        return Ok(run_tui(start_dir, config));
    }

    if !arg_path.exists() {
        Err(ResolveError::NotFound(arg_path))
    } else {
        Err(ResolveError::NotADirectory(arg_path))
    }
}

/// 有効な最深ディレクトリを親方向に探索
fn find_deepest_dir(path: &path::Path) -> Option<path::PathBuf> {
    let mut current = path.to_path_buf();
    while !current.is_dir() {
        if !current.pop() {
            return None;
        }
    }
    Some(current)
}

/// TUI を起動して選択されたディレクトリを取得
fn run_tui(start_dir: path::PathBuf, config: config::Config) -> path::PathBuf {
    match tui::path_finder(start_dir, fs::EntryType::Dir, config) {
        Ok(Some(dir)) => dir,
        Ok(None) => std::process::exit(0),
        Err(e) => {
            eprintln!("TUI Error: {}", e);
            std::process::exit(1);
        }
    }
}

/// カレントディレクトリの取得（失敗時はエラー終了）
fn get_current_dir() -> path::PathBuf {
    match std::env::current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Error getting current directory: {}", e);
            std::process::exit(1);
        }
    }
}

/// init サブコマンドの処理
fn handle_init(shell_str: &str) -> ! {
    match shell::Shell::from_str(shell_str) {
        Ok(sh) => {
            print!("{}", shell::get_init_script(sh));
            std::process::exit(0);
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            eprintln!("Usage: cdx init <bash|zsh|fish>");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_test_dir(name: &str) -> path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("cdx_main_test_{}_{}", name, nanos));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_find_deepest_dir_existing_dir() {
        let dir = temp_test_dir("existing");
        let result = find_deepest_dir(&dir);
        assert_eq!(result, Some(dir.clone()));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_find_deepest_dir_nested_nonexistent() {
        let dir = temp_test_dir("nested");
        let nonexistent = dir.join("non_existent_1/non_existent_2");
        let result = find_deepest_dir(&nonexistent);
        assert_eq!(result, Some(dir.clone()));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_find_deepest_dir_with_file_in_path() {
        let dir = temp_test_dir("file_in_path");
        let file_path = dir.join("some_file.txt");
        File::create(&file_path).unwrap();

        // ファイルおよびその配下を指定した時、親ディレクトリまで遡ること
        let invalid_path = file_path.join("child");
        let result = find_deepest_dir(&invalid_path);
        assert_eq!(result, Some(dir.clone()));

        // ファイルそのものを指定した場合も親ディレクトリまで遡ること
        let file_result = find_deepest_dir(&file_path);
        assert_eq!(file_result, Some(dir.clone()));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_find_deepest_dir_nonexistent_relative() {
        let path = path::Path::new("totally_nonexistent_dir_cdx_test_xyz/sub");
        let result = find_deepest_dir(path);
        assert_eq!(result, None);
    }

    #[test]
    fn test_resolve_target_dir_existing_dir() {
        let dir = temp_test_dir("resolve_existing");
        let config = config::Config::default();
        let result = resolve_target_dir(dir.to_str().unwrap(), config);
        assert_eq!(result, Ok(dir.clone()));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_resolve_target_dir_fallback_disabled() {
        let mut config = config::Config::default();
        config.system.partial_navigation_fallback = false;

        let nonexistent = "nonexistent_test_path_12345";
        let result = resolve_target_dir(nonexistent, config);
        assert_eq!(
            result,
            Err(ResolveError::NotFound(path::PathBuf::from(nonexistent)))
        );
    }

    #[test]
    fn test_resolve_target_dir_not_a_directory() {
        let dir = temp_test_dir("resolve_file");
        let file_path = dir.join("file.txt");
        File::create(&file_path).unwrap();

        let mut config = config::Config::default();
        config.system.partial_navigation_fallback = false;

        let result = resolve_target_dir(file_path.to_str().unwrap(), config);
        assert_eq!(result, Err(ResolveError::NotADirectory(file_path)));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_resolve_error_display() {
        let not_found = ResolveError::NotFound(path::PathBuf::from("foo/bar"));
        assert_eq!(format!("{}", not_found), "'foo/bar' does not exist.");

        let not_dir = ResolveError::NotADirectory(path::PathBuf::from("foo/file.txt"));
        assert_eq!(format!("{}", not_dir), "'foo/file.txt' is not a directory.");
    }
}
