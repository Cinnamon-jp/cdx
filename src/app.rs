// Copyright (c) 2026 Cinnamon-jp
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use std::io;
use std::path::PathBuf;

use crate::config::Config;
use crate::fs::{EntryType, get_entries, try_get_entries};

pub struct App {
    pub current_dir: PathBuf,
    pub selected: usize,
    pub input: String,
    pub all_entries: Vec<String>,
    pub filtered_entries: Vec<String>,
    pub entry_type: EntryType,
    pub config: Config,
}

impl App {
    pub fn new(current_dir: PathBuf, entry_type: EntryType, config: Config) -> io::Result<Self> {
        let all_entries = get_entries(&current_dir, entry_type)?;
        let mut app = Self {
            current_dir,
            selected: 0,
            input: String::new(),
            filtered_entries: all_entries.clone(),
            all_entries,
            entry_type,
            config,
        };
        app.update_filter();
        Ok(app)
    }

    /// 入力文字列に基づいてエントリを絞り込み、選択インデックスを範囲内に収める
    pub fn update_filter(&mut self) {
        let case_insensitive = self.config.system.use_case_insensitive_search;
        let search_query = if case_insensitive {
            self.input.to_lowercase()
        } else {
            self.input.clone()
        };

        self.filtered_entries = self
            .all_entries
            .iter()
            .filter(|s| {
                if case_insensitive {
                    s.to_lowercase().starts_with(&search_query)
                } else {
                    s.starts_with(&search_query)
                }
            })
            .cloned()
            .collect();

        self.selected = self
            .selected
            .min(self.filtered_entries.len().saturating_sub(1));
    }

    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn move_down(&mut self) {
        if !self.filtered_entries.is_empty() {
            self.selected = self
                .selected
                .saturating_add(1)
                .min(self.filtered_entries.len() - 1);
        }
    }

    pub fn push_input(&mut self, ch: char) {
        self.input.push(ch);
        self.update_filter();
    }

    pub fn pop_input(&mut self) -> io::Result<()> {
        if self.input.is_empty() {
            let prev_dir = self.current_dir.clone();
            self.current_dir.pop();
            self.all_entries = try_get_entries(&mut self.current_dir, &prev_dir, self.entry_type)?;
            self.selected = 0;
            self.update_filter();
        } else {
            self.input.pop();
            self.update_filter();
        }
        Ok(())
    }

    pub fn handle_tab(&mut self) -> io::Result<()> {
        if self.filtered_entries.is_empty() {
            return Ok(());
        }

        let selected_element = &self.filtered_entries[self.selected];
        let prev_dir = self.current_dir.clone();

        match selected_element.as_str() {
            "." => {} // カレントディレクトリを維持
            ".." => {
                self.current_dir.pop();
            }
            _ => {
                let next_path = self.current_dir.join(selected_element);
                if next_path.is_dir() {
                    self.current_dir.push(selected_element);
                }
            }
        }

        self.all_entries = try_get_entries(&mut self.current_dir, &prev_dir, self.entry_type)?;
        self.input.clear();
        self.selected = 0;
        self.update_filter();
        Ok(())
    }

    /// Enterキー入力を処理（ディレクトリまたはファイルパス確定時は `Some(PathBuf)` を返す）
    pub fn handle_enter(&self) -> Option<PathBuf> {
        if self.filtered_entries.is_empty() {
            return None;
        }

        let selected_element = &self.filtered_entries[self.selected];

        let target_path = match selected_element.as_str() {
            "." => self.current_dir.clone(),
            ".." => self
                .current_dir
                .parent()
                .unwrap_or(&self.current_dir)
                .to_path_buf(),
            _ => self.current_dir.join(selected_element),
        };

        Some(target_path)
    }

    /// 描画対象となるエントリの表示範囲（開始・終了インデックス）を計算
    pub fn visible_range(&self, list_rows: usize) -> (usize, usize) {
        let start_idx = if self.filtered_entries.is_empty() || list_rows == 0 {
            0
        } else if self.selected >= list_rows / 2 {
            std::cmp::min(
                self.selected - list_rows / 2,
                self.filtered_entries.len().saturating_sub(list_rows),
            )
        } else {
            0
        };

        let end_idx = std::cmp::min(self.filtered_entries.len(), start_idx + list_rows);
        (start_idx, end_idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_test_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("cdx_app_test_{}_{}", name, nanos));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_app_filter_case_insensitive() -> io::Result<()> {
        let dir = temp_test_dir("filter_ci");
        fs::create_dir(dir.join("Alpha"))?;
        fs::create_dir(dir.join("beta"))?;

        let mut app = App::new(dir.clone(), EntryType::Dir, Config::default())?;
        assert!(app.filtered_entries.contains(&"Alpha".to_string()));
        assert!(app.filtered_entries.contains(&"beta".to_string()));

        app.push_input('a');
        assert!(app.filtered_entries.contains(&"Alpha".to_string()));
        assert!(!app.filtered_entries.contains(&"beta".to_string()));

        let _ = fs::remove_dir_all(dir);
        Ok(())
    }

    #[test]
    fn test_app_filter_case_sensitive() -> io::Result<()> {
        let dir = temp_test_dir("filter_cs");
        fs::create_dir(dir.join("Alpha"))?;
        fs::create_dir(dir.join("alpha_small"))?;

        let mut config = Config::default();
        config.system.use_case_insensitive_search = false;

        let mut app = App::new(dir.clone(), EntryType::Dir, config)?;
        app.push_input('a');
        assert!(!app.filtered_entries.contains(&"Alpha".to_string()));
        assert!(app.filtered_entries.contains(&"alpha_small".to_string()));

        let _ = fs::remove_dir_all(dir);
        Ok(())
    }

    #[test]
    fn test_app_navigation_and_visible_range() -> io::Result<()> {
        let dir = temp_test_dir("nav");
        for i in 0..10 {
            fs::create_dir(dir.join(format!("dir_{:02}", i)))?;
        }

        let mut app = App::new(dir.clone(), EntryType::Dir, Config::default())?;
        assert_eq!(app.selected, 0);

        app.move_down();
        assert_eq!(app.selected, 1);

        app.move_up();
        assert_eq!(app.selected, 0);

        let (start, end) = app.visible_range(5);
        assert_eq!(start, 0);
        assert_eq!(end, 5);

        let _ = fs::remove_dir_all(dir);
        Ok(())
    }

    #[test]
    fn test_app_enter_confirm_dot() -> io::Result<()> {
        let dir = temp_test_dir("enter_dot");
        let mut app = App::new(dir.clone(), EntryType::Dir, Config::default())?;

        // "." を検索して選択
        let dot_pos = app.filtered_entries.iter().position(|e| e == ".").unwrap();
        app.selected = dot_pos;

        let result = app.handle_enter();
        assert_eq!(result, Some(dir.clone()));

        let _ = fs::remove_dir_all(dir);
        Ok(())
    }

    #[test]
    fn test_app_enter_confirm_entry() -> io::Result<()> {
        let dir = temp_test_dir("enter_entry");
        fs::create_dir(dir.join("child_dir"))?;

        let mut app = App::new(dir.clone(), EntryType::Dir, Config::default())?;

        // "child_dir" を検索して選択
        let child_pos = app
            .filtered_entries
            .iter()
            .position(|e| e == "child_dir")
            .unwrap();
        app.selected = child_pos;

        let result = app.handle_enter();
        assert_eq!(result, Some(dir.join("child_dir")));

        let _ = fs::remove_dir_all(dir);
        Ok(())
    }

    #[test]
    fn test_app_enter_confirm_dotdot() -> io::Result<()> {
        let parent = temp_test_dir("enter_dotdot_parent");
        let child = parent.join("child");
        fs::create_dir(&child)?;

        let mut app = App::new(child.clone(), EntryType::Dir, Config::default())?;

        // ".." を検索して選択
        let dotdot_pos = app.filtered_entries.iter().position(|e| e == "..").unwrap();
        app.selected = dotdot_pos;

        let result = app.handle_enter();
        assert_eq!(result, Some(parent.clone()));

        let _ = fs::remove_dir_all(parent);
        Ok(())
    }
}
