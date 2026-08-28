// Copyright (c) 2026 Cinnamon-jp
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use crossterm::{
    cursor, queue,
    style::{Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use std::io::{self, Write};

use crate::app::App;
use crate::fs::EntryType;

/// 画面全体を描画
pub fn render(stderr: &mut io::Stderr, app: &App) -> io::Result<()> {
    let (_, rows) = crossterm::terminal::size()?;
    let list_rows = rows.saturating_sub(4) as usize;

    let (start_idx, end_idx) = app.visible_range(list_rows);

    // パスを整形（ルートディレクトリでの "//" を防止）
    let display_path = app.current_dir.display().to_string();
    let separator = if display_path.ends_with('/') { "" } else { "/" };

    // 画面をクリアしてパスと入力行を描画
    queue!(
        stderr,
        Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        SetForegroundColor(app.config.ui.path_foreground_color),
        Print(format!("{}{}{}\r\n", display_path, separator, app.input)),
        ResetColor,
    )?;

    // エントリ一覧を描画
    if app.filtered_entries.is_empty() {
        queue!(stderr, Print("  (Empty)\r\n"))?;
    } else {
        for (i, name) in app
            .filtered_entries
            .iter()
            .enumerate()
            .take(end_idx)
            .skip(start_idx)
        {
            // Fileモード時: ディレクトリの末尾に "/" を付与
            let display_name = if app.entry_type == EntryType::File
                && *name != ".."
                && app.current_dir.join(name).is_dir()
            {
                format!("{}/", name)
            } else {
                name.clone()
            };

            if i == app.selected {
                queue!(
                    stderr,
                    SetForegroundColor(app.config.ui.selected_foreground_color),
                    SetBackgroundColor(app.config.ui.selected_background_color),
                    Print(format!("{}\r\n", display_name)),
                    ResetColor,
                )?;
            } else {
                queue!(stderr, Print(format!("{}\r\n", display_name)))?;
            }
        }
    }

    stderr.flush()?;
    Ok(())
}
