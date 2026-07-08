mod tui;

use std::{env, path};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        eprintln!("Usage: cdx <directory>");
        std::process::exit(1);
    }

    let target_dir: path::PathBuf;

    if args.len() == 1 {
        match tui::path_finder() {
            Ok(Some(dir)) => target_dir = path::PathBuf::from(dir),
            Ok(None) => std::process::exit(0),
            Err(e) => {
                eprintln!("TUI Error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        target_dir = path::PathBuf::from(&args[1]);
    }

    if !target_dir.exists() {
        eprintln!("Error: '{}' does not exist.", target_dir.display());
        std::process::exit(1);
    }

    if !target_dir.is_dir() {
        eprintln!("Error: '{}' is not a directory.", target_dir.display());
        std::process::exit(1);
    }

    match target_dir.canonicalize() {
        Ok(abs_path) => println!("{}", abs_path.display()),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
