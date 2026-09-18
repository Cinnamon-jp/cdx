// Copyright (c) 2026 Cinnamon-jp
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::path::PathBuf;
use std::io;

use crate::app::App;
use crate::config::Config;
use crate::fs::EntryType;
use crate::ui;

// 公開関数: 端末の初期化と復元を保証するラッパー
pub fn path_finder(
    current_dir: PathBuf,
    entry_type: EntryType,
    config: Config,
) -> io::Result<Option<PathBuf>> {
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, cursor::Hide)?;

    let result = path_finder_inner(&mut stderr, current_dir, entry_type, config);

    // 処理結果に関わらず必ず端末の状態を復元
    execute!(stderr, cursor::Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;

    result
}

// 内部ロジック（イベントループと描画制御）
fn path_finder_inner(
    stderr: &mut io::Stderr,
    current_dir: PathBuf,
    entry_type: EntryType,
    config: Config,
) -> io::Result<Option<PathBuf>> {
    let mut app = App::new(current_dir, entry_type, config)?;

    loop {
        ui::render(stderr, &app)?;

        // キー入力を処理（ブロッキング待機）
        match event::read()? {
            Event::Key(key) => {
                // Ctrl+C は状況を問わず即座に終了
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    return Ok(None);
                }

                // リストが空の場合のキー処理
                if app.filtered_entries.is_empty() {
                    match key.code {
                        KeyCode::Esc => return Ok(None),
                        KeyCode::Char(ch) => app.push_input(ch),
                        KeyCode::Backspace => app.pop_input()?,
                        _ => {}
                    }
                    continue;
                }

                // リストが空でない場合のキー処理
                match key.code {
                    KeyCode::Esc => return Ok(None),
                    KeyCode::Up => app.move_up(),
                    KeyCode::Down => app.move_down(),
                    KeyCode::Tab => app.handle_tab()?,
                    KeyCode::Backspace | KeyCode::BackTab => app.pop_input()?,
                    KeyCode::Enter => {
                        if let Some(selected_path) = app.handle_enter() {
                            return Ok(Some(selected_path));
                        }
                    }
                    KeyCode::Char(ch) => app.push_input(ch),
                    _ => {}
                }
            }
            Event::Resize(_, _) => {
                // 何もせずループを継続（自動的に再描画）
                continue;
            }
            _ => {}
        }
    }
}
