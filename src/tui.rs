use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode},
    execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use std::io::{self, Write};
use std::path::Path;
// use std::str::FromStr;
use std::fs;

enum EntryType {
    Dir,
    // File,
}

// impl FromStr for EntryType {
//     type Err = String;
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s {
//             "dir" => Ok(EntryType::Dir),
//             "file" => Ok(EntryType::File),
//             _ => Err(format!(
//                 "Invalid entry type: '{}'. Expected 'dir' or 'file'.",
//                 s
//             )),
//         }
//     }
// }

// ディレクトリ下のファイル・ディレクトリのリストを取得する関数
fn get_entries(dir: &Path, target: EntryType) -> io::Result<Vec<String>> {
    let mut items = Vec::new();
    let is_only_dirs = matches!(target, EntryType::Dir);
    let entries = fs::read_dir(dir)?;

    for entry in entries.flatten() {
        let path = entry.path();
        if is_only_dirs && !path.is_dir() {
            continue;
        }
        if let Some(name) = path.file_name() {
            items.push(name.to_string_lossy().into_owned());
        }
    }
    items.push(".".to_string());
    items.push("..".to_string());
    items.sort();
    Ok(items)
}

pub fn path_finder() -> io::Result<Option<String>> {
    // <<< ターミナル初期設定
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;
    // >>> ターミナル初期設定

    // 情報保持変数
    let mut current_dir = std::env::current_dir()?;
    let mut selected: usize = 0; // 選択しているインデックス
    let mut path_input = String::new(); // 検索する文字列
    let mut all_dirs = get_entries(&current_dir, EntryType::Dir)?; // 初回のディレクトリ一覧取得

    // 入力検知・描画ループ
    loop {
        // <<< ターミナル情報取得
        let (_, rows) = crossterm::terminal::size()?; // ターミナルの行数を取得
        let list_rows = rows.saturating_sub(4) as usize; // エントリ表示可能行数を計算
        // >>> ターミナル情報取得

        // ディレクトリ内を検索
        let target_dirs: Vec<String> = all_dirs
            .iter()
            .filter(|s| s.starts_with(&path_input))
            .cloned()
            .collect();

        // 検索を踏まえて選択要素を更新
        selected = selected.min(target_dirs.len().saturating_sub(1)); // 自身の値と比較

        // 描画開始インデックスを計算
        let start_idx = if target_dirs.is_empty() || list_rows == 0 {
            0
        } else if selected >= list_rows / 2 {
            // 選択部分が画面の下半分に入った場合の処理
            std::cmp::min(
                selected - list_rows / 2,
                target_dirs.len().saturating_sub(list_rows),
            )
        } else {
            0
        };

        // 描画終了インデックスを計算
        let end_idx = std::cmp::min(
            target_dirs.len(),
            start_idx + list_rows
        );

        // 描画キューに追加
        queue!(
            stdout,
            Clear(ClearType::All),
            MoveTo(0, 0),
            Print(format!("{}/{}\r\n", current_dir.display(), path_input)),
        )?;

        // ディレクトリの描画
        if target_dirs.is_empty() {
            queue!(stdout, Print("  (Empty)\r\n"))?;
        } else {
            for i in start_idx..end_idx {
                let dir = &target_dirs[i];
                if i == selected {
                    queue!(
                        stdout,
                        SetForegroundColor(Color::Cyan),
                        SetBackgroundColor(Color::DarkGrey),
                        Print(format!("{}\r\n", dir)),
                        ResetColor,
                    )?;
                } else {
                    queue!(stdout, Print(format!("{}\r\n", dir)))?;
                }
            }
        }

        // キューを実行
        stdout.flush()?;

        // キー入力を処理 (ブロックして待機)
        match event::read()? {
            Event::Key(key) => {
                // リストが空の場合: Esc / 文字入力 / Backspace のみ処理
                if target_dirs.is_empty() {
                    match key.code {
                        KeyCode::Esc => {
                            execute!(stdout, Show, LeaveAlternateScreen)?;
                            disable_raw_mode()?;
                            return Ok(None);
                        }
                        KeyCode::Char(ch) => path_input.push(ch),
                        KeyCode::Backspace => {
                            if path_input.is_empty() {
                                current_dir.pop();
                                all_dirs = get_entries(&current_dir, EntryType::Dir)?;
                            } else {
                                path_input.pop();
                            }
                        }
                        _ => {}
                    }
                    continue;
                }

                // ここ以降は target_dirs が非空であることが保証される
                match key.code {
                    KeyCode::Esc => {
                        execute!(stdout, Show, LeaveAlternateScreen)?;
                        disable_raw_mode()?;
                        return Ok(None);
                    }
                    KeyCode::Up => selected = selected.saturating_sub(1),
                    KeyCode::Down => {
                        selected = selected.saturating_add(1).min(target_dirs.len() - 1)
                    }
                    KeyCode::Tab => {
                        let selected_element = &target_dirs[selected];

                        if selected_element == "." {
                        } else if selected_element == ".." {
                            current_dir.pop();
                        } else {
                            current_dir.push(selected_element);
                        }

                        // ディレクトリが移動したので一覧を再取得
                        all_dirs = get_entries(&current_dir, EntryType::Dir)?;

                        // 各種パラメータ初期化
                        path_input = String::new();
                        selected = 0;
                    }
                    KeyCode::Backspace => {
                        if path_input.is_empty() {
                            current_dir.pop();

                            // ディレクトリが移動したので一覧を再取得
                            all_dirs = get_entries(&current_dir, EntryType::Dir)?;
                        } else {
                            path_input.pop();
                        }
                    }
                    KeyCode::Enter => {
                        let selected_element = &target_dirs[selected];
                        let result_dir = if selected_element == "." {
                            current_dir.clone()
                        } else if selected_element == ".." {
                            current_dir.pop();
                            current_dir.clone()
                        } else {
                            current_dir.join(selected_element)
                        };
                        execute!(stdout, Show, LeaveAlternateScreen)?;
                        disable_raw_mode()?;
                        return Ok(Some(result_dir.to_string_lossy().into_owned()));
                    }
                    // KeyCode::Left | KeyCode::Char('h') => {
                    //     if let Some(parent) = current_dir.parent() {
                    //         current_dir = parent.to_path_buf();
                    //         target_dirs = get_entries(&current_dir, EntryType::Dir)?;
                    //         selected = 0;
                    //     }
                    // }
                    // KeyCode::Right | KeyCode::Char('l') => {
                    //     if !target_dirs.is_empty() {
                    //         current_dir.push(&target_dirs[selected]);
                    //         target_dirs = get_entries(&current_dir, EntryType::Dir)?;
                    //         selected = 0;
                    //     }
                    // }
                    KeyCode::Char(ch) => {
                        path_input.push(ch);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
