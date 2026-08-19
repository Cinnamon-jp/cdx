// Copyright (c) 2026 Cinnamon-jp
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyModifiers},
    execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    Dir,
    File,
}

// 公開関数: ターミナルの初期化と復帰を保証するラッパー
pub fn path_finder(entry_type: EntryType) -> io::Result<Option<PathBuf>> {
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, Hide)?;

    let result = path_finder_inner(&mut stderr, entry_type);

    // result が Ok でも Err でも必ず復帰処理を実行
    execute!(stderr, Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;

    result
}

// ディレクトリ下のファイル・ディレクトリのリストを取得する関数
fn get_entries(dir: &Path, target: EntryType) -> io::Result<Vec<String>> {
    let mut items = Vec::new();
    let is_only_dirs = matches!(target, EntryType::Dir);
    let entries = fs::read_dir(dir)?;

    for entry in entries.flatten() {
        if is_only_dirs {
            // file_type() は readdir のキャッシュを利用(stat 不要）
            // シンボリックリンクの場合のみ path.is_dir() にフォールバック
            if let Ok(ft) = entry.file_type() {
                if ft.is_symlink() {
                    // シンボリックリンク → リンク先がディレクトリか確認(stat 必要）
                    if !entry.path().is_dir() {
                        continue;
                    }
                } else if !ft.is_dir() {
                    continue;
                }
            } else {
                continue; // file_type 取得失敗時はスキップ
            }
        }
        if let Some(name) = entry.file_name().to_str() {
            items.push(name.to_owned());
        }
    }
    // Dir モードのみ "." を追加（カレントディレクトリ確定用）
    if matches!(target, EntryType::Dir) {
        items.push(".".to_string());
    }
    // ルートディレクトリでなければ ".." を追加
    if dir.parent().is_some() {
        items.push("..".to_string());
    }
    items.sort();
    Ok(items)
}

/// get_entries を安全に呼ぶヘルパー。失敗時は current_dir を prev_dir に戻す。
fn try_get_entries(
    current_dir: &mut PathBuf,
    prev_dir: &Path,
    target: EntryType,
) -> io::Result<Vec<String>> {
    match get_entries(current_dir, target) {
        Ok(entries) => Ok(entries),
        Err(_) => {
            // アクセスできないので前のディレクトリに戻す
            *current_dir = prev_dir.to_path_buf();
            get_entries(current_dir, target)
        }
    }
}

// 内部ロジック
fn path_finder_inner(
    stderr: &mut io::Stderr,
    entry_type: EntryType,
) -> io::Result<Option<PathBuf>> {
    // 情報保持変数
    let mut current_dir = std::env::current_dir()?;
    let mut selected: usize = 0; // 選択しているインデックス
    let mut path_input = String::new(); // 検索する文字列
    let mut all_entries = get_entries(&current_dir, entry_type)?; // 初回のエントリ一覧取得

    // 入力検知・描画ループ
    loop {
        // ターミナル情報取得
        let (_, rows) = crossterm::terminal::size()?; // ターミナルの行数を取得
        let list_rows = rows.saturating_sub(4) as usize; // エントリ表示可能行数を計算

        // ディレクトリ内を検索
        let filtered_entries: Vec<String> = all_entries
            .iter()
            .filter(|s| s.to_lowercase().starts_with(&path_input.to_lowercase()))
            .cloned()
            .collect();

        // 検索を踏まえて選択要素を更新
        selected = selected.min(filtered_entries.len().saturating_sub(1)); // 自身の値と比較

        // 描画開始インデックスを計算
        let start_idx = if filtered_entries.is_empty() || list_rows == 0 {
            0
        } else if selected >= list_rows / 2 {
            // 選択部分が画面の下半分に入った場合の処理
            std::cmp::min(
                selected - list_rows / 2,
                filtered_entries.len().saturating_sub(list_rows),
            )
        } else {
            0
        };

        // 描画終了インデックスを計算
        let end_idx = std::cmp::min(filtered_entries.len(), start_idx + list_rows);

        // パス表示（ルートディレクトリ時の "//" を防止）
        let display_path = current_dir.display().to_string();
        let separator = if display_path.ends_with('/') { "" } else { "/" };

        // 描画キューに追加
        queue!(
            stderr,
            Clear(ClearType::All),
            MoveTo(0, 0),
            Print(format!("{}{}{}\r\n", display_path, separator, path_input)),
        )?;

        // エントリの描画
        if filtered_entries.is_empty() {
            queue!(stderr, Print("  (Empty)\r\n"))?;
        } else {
            for (i, name) in filtered_entries
                .iter()
                .enumerate()
                .take(end_idx)
                .skip(start_idx)
            {
                // File モード時: ディレクトリには "/" を付ける
                let display_name = if entry_type == EntryType::File
                    && *name != ".."
                    && current_dir.join(name).is_dir()
                {
                    format!("{}/", name)
                } else {
                    name.clone()
                };

                if i == selected {
                    queue!(
                        stderr,
                        SetForegroundColor(Color::Cyan),
                        SetBackgroundColor(Color::DarkGrey),
                        Print(format!("{}\r\n", display_name)),
                        ResetColor,
                    )?;
                } else {
                    queue!(stderr, Print(format!("{}\r\n", display_name)))?;
                }
            }
        }

        // キューを実行
        stderr.flush()?;

        // キー入力を処理 (ブロックして待機)
        match event::read()? {
            Event::Key(key) => {
                // Ctrl+C は全状況で即終了
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    return Ok(None);
                }

                // リストが空の場合: Esc / 文字入力 / Backspace のみ処理
                if filtered_entries.is_empty() {
                    match key.code {
                        KeyCode::Esc => {
                            return Ok(None);
                        }
                        KeyCode::Char(ch) => path_input.push(ch),
                        KeyCode::Backspace => {
                            if path_input.is_empty() {
                                let prev_dir = current_dir.clone();
                                current_dir.pop();
                                all_entries =
                                    try_get_entries(&mut current_dir, &prev_dir, entry_type)?;
                                selected = 0;
                            } else {
                                path_input.pop();
                            }
                        }
                        _ => {}
                    }
                    continue;
                }

                // ここ以降は filtered_entries が非空であることが保証される
                match key.code {
                    KeyCode::Esc => {
                        return Ok(None);
                    }
                    KeyCode::Up => selected = selected.saturating_sub(1),
                    KeyCode::Down => {
                        selected = selected.saturating_add(1).min(filtered_entries.len() - 1)
                    }
                    KeyCode::Tab => {
                        let selected_element = &filtered_entries[selected];
                        let prev_dir = current_dir.clone();

                        if selected_element == "." {
                        } else if selected_element == ".." {
                            current_dir.pop();
                        } else {
                            let next_path = current_dir.join(selected_element);
                            if next_path.is_dir() {
                                current_dir.push(selected_element);
                            }
                        }

                        // ディレクトリが移動したので一覧を再取得
                        all_entries = try_get_entries(&mut current_dir, &prev_dir, entry_type)?;

                        // 各種パラメータ初期化
                        path_input.clear();
                        selected = 0;
                    }
                    KeyCode::Backspace | KeyCode::BackTab => {
                        if path_input.is_empty() {
                            let prev_dir = current_dir.clone();
                            current_dir.pop();

                            // ディレクトリが移動したので一覧を再取得
                            all_entries = try_get_entries(&mut current_dir, &prev_dir, entry_type)?;
                            path_input.clear();
                            selected = 0;
                        } else {
                            path_input.pop();
                        }
                    }
                    KeyCode::Enter => {
                        let selected_element = filtered_entries[selected].clone();
                        // . を選択したときだけ終了(Dir モードのみ表示される)
                        if selected_element == "." {
                            return Ok(Some(current_dir.clone()));
                        } else if selected_element == ".." {
                            let prev_dir = current_dir.clone();
                            current_dir.pop();
                            all_entries = try_get_entries(&mut current_dir, &prev_dir, entry_type)?;
                        } else {
                            let next_path = current_dir.join(&selected_element);
                            if next_path.is_dir() {
                                let prev_dir = current_dir.clone();
                                current_dir.push(&selected_element);
                                all_entries =
                                    try_get_entries(&mut current_dir, &prev_dir, entry_type)?;
                            } else {
                                return Ok(Some(next_path));
                            }
                        };

                        path_input.clear();
                        selected = 0;
                    }
                    KeyCode::Char(ch) => {
                        path_input.push(ch);
                    }
                    _ => {}
                }
            }
            Event::Resize(_, _) => {
                // 何もせずループ先頭に戻る → 自動的に再描画される
                continue;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_test_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("cdx_tui_test_{}_{}", name, nanos));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_get_entries_dir_only() -> io::Result<()> {
        let dir = temp_test_dir("dir_only");
        fs::create_dir(dir.join("sub_dir"))?;
        File::create(dir.join("test_file.txt"))?;

        let entries = get_entries(&dir, EntryType::Dir)?;
        assert!(entries.contains(&".".to_string()));
        assert!(entries.contains(&"..".to_string()));
        assert!(entries.contains(&"sub_dir".to_string()));
        assert!(!entries.contains(&"test_file.txt".to_string()));

        let _ = fs::remove_dir_all(dir);
        Ok(())
    }

    #[test]
    fn test_get_entries_file_and_dir() -> io::Result<()> {
        let dir = temp_test_dir("file_and_dir");
        fs::create_dir(dir.join("sub_dir"))?;
        File::create(dir.join("test_file.txt"))?;

        let entries = get_entries(&dir, EntryType::File)?;
        assert!(!entries.contains(&".".to_string()));
        assert!(entries.contains(&"..".to_string()));
        assert!(entries.contains(&"sub_dir".to_string()));
        assert!(entries.contains(&"test_file.txt".to_string()));

        let _ = fs::remove_dir_all(dir);
        Ok(())
    }
}
