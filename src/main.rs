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

    if args.get(1).is_some_and(|s| s == "init") && args.len() >= 3 {
        if args.len() == 3 {
            match shell::Shell::from_str(&args[2]) {
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
        } else {
            eprintln!("Usage: cdx init <bash|zsh|fish>");
            std::process::exit(1);
        }
    }

    if args.len() > 2 {
        eprintln!("Usage: cdx [directory] | cdx init <shell>");
        std::process::exit(1);
    }

    let target_dir: path::PathBuf;
    let config = config::load_config_or_default();

    // 引数に応じてTUIモードとダイレクトモードを切り替え
    if args.len() == 1 {
        let current_dir = match std::env::current_dir() {
            Ok(dir) => dir,
            Err(e) => {
                eprintln!("Error getting current directory: {}", e);
                std::process::exit(1);
            }
        };
        match tui::path_finder(current_dir, fs::EntryType::Dir, config) {
            Ok(Some(dir)) => target_dir = dir,
            Ok(None) => std::process::exit(0),
            Err(e) => {
                eprintln!("TUI Error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        if config.system.partial_navigation_fallback {
            let arg_path = path::PathBuf::from(&args[1]);
            if arg_path.is_dir() {
                target_dir = arg_path;
            } else {
                let mut exist_path = arg_path.clone();

                while !exist_path.exists() {
                    if !exist_path.pop() {
                        break;
                    }
                }
            }
            
            // `exist_path.pop();` が一回でも実行された場合、tui::path_finderを起動する
            if exist_path != arg_path {
                match tui::path_finder(exist_path, fs::EntryType::Dir, config) {
                    Ok(Some(dir)) => target_dir = dir,
                    Ok(None) => std::process::exit(0),
                    Err(e) => {
                        eprintln!("TUI Error: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                target_dir = exist_path;
            }
        } else {
            target_dir = path::PathBuf::from(&args[1]);
        };
    }

    // <<< Error handling
    if !target_dir.exists() {
        eprintln!("Error: '{}' does not exist.", target_dir.display());
        if args.get(1).is_some_and(|s| s == "init") {
            eprintln!("(Note: To initialize shell integration, run 'cdx init <bash|zsh|fish>')");
        }
        std::process::exit(1);
    }
    if !target_dir.is_dir() {
        eprintln!("Error: '{}' is not a directory.", target_dir.display());
        std::process::exit(1);
    }
    // >>> Error handling

    // Convert to absolute path
    match target_dir.canonicalize() {
        Ok(abs_path) => println!("{}", abs_path.display()),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
