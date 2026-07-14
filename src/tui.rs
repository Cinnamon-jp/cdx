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
use std::path::Path;

enum EntryType {
    Dir,
    // File,
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
    items.push(".".to_string());
    items.push("..".to_string());
    items.sort();
    Ok(items)
}

// 公開関数: ターミナルの初期化と復帰を保証するラッパー
pub fn path_finder() -> io::Result<Option<String>> {
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, Hide)?;

    let result = path_finder_inner(&mut stderr);

    // result が Ok でも Err でも必ず復帰処理を実行
    execute!(stderr, Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;

    result
}

// 内部ロジック: エラー時は ? で即座に返しても安全
fn path_finder_inner(stderr: &mut io::Stderr) -> io::Result<Option<String>> {
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
        let end_idx = std::cmp::min(target_dirs.len(), start_idx + list_rows);

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

        // ディレクトリの描画
        if target_dirs.is_empty() {
            queue!(stderr, Print("  (Empty)\r\n"))?;
        } else {
            for (i, dir) in target_dirs.iter().enumerate().take(end_idx).skip(start_idx) {
                if i == selected {
                    queue!(
                        stderr,
                        SetForegroundColor(Color::Cyan),
                        SetBackgroundColor(Color::DarkGrey),
                        Print(format!("{}\r\n", dir)),
                        ResetColor,
                    )?;
                } else {
                    queue!(stderr, Print(format!("{}\r\n", dir)))?;
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
                if target_dirs.is_empty() {
                    match key.code {
                        KeyCode::Esc => {
                            return Ok(None);
                        }
                        KeyCode::Char(ch) => path_input.push(ch),
                        KeyCode::Backspace => {
                            if path_input.is_empty() {
                                current_dir.pop();
                                all_dirs = get_entries(&current_dir, EntryType::Dir)?;
                                selected = 0;
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
                        path_input.clear();
                        selected = 0;
                    }
                    KeyCode::Backspace | KeyCode::BackTab => {
                        if path_input.is_empty() {
                            current_dir.pop();

                            // ディレクトリが移動したので一覧を再取得
                            all_dirs = get_entries(&current_dir, EntryType::Dir)?;
                            path_input.clear();
                            selected = 0;
                        } else {
                            path_input.pop();
                        }
                    }
                    KeyCode::Enter => {
                        let selected_element = &target_dirs[selected];
                        // . を選択したときだけ終了
                        if selected_element == "." {
                            return Ok(Some(current_dir.to_string_lossy().into_owned()));
                        } else if selected_element == ".." {
                            current_dir.pop();
                        } else {
                            current_dir.push(selected_element);
                        };

                        // ディレクトリが移動したので一覧を再取得
                        all_dirs = get_entries(&current_dir, EntryType::Dir)?;
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
