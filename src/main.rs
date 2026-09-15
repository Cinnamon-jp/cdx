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

    // 引数に応じてTUIモードとダイレクトモードを切り替え
    if args.len() == 1 {
        let config = config::load_config_or_default();
        match tui::path_finder(fs::EntryType::Dir, config) {
            Ok(Some(dir)) => target_dir = dir,
            Ok(None) => std::process::exit(0),
            Err(e) => {
                eprintln!("TUI Error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        target_dir = path::PathBuf::from(&args[1]);
    }

    // <<< エラーハンドリング
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
    // >>> エラーハンドリング

    // 絶対パスに変換
    match target_dir.canonicalize() {
        Ok(abs_path) => println!("{}", abs_path.display()),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
