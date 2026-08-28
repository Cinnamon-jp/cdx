// Copyright (c) 2026 Cinnamon-jp
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    Dir,
    File,
}

/// ディレクトリ直下のファイル・ディレクトリの一覧を取得
pub fn get_entries(dir: &Path, target: EntryType) -> io::Result<Vec<String>> {
    let mut items = Vec::new();
    let is_only_dirs = matches!(target, EntryType::Dir);
    let entries = fs::read_dir(dir)?;

    for entry in entries.flatten() {
        if is_only_dirs {
            // file_type() は readdir のキャッシュを利用（stat 不要）
            // シンボリックリンクの場合のみ path.is_dir() にフォールバック
            if let Ok(ft) = entry.file_type() {
                if ft.is_symlink() {
                    // シンボリックリンク → リンク先がディレクトリか確認（stat 必要）
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

/// get_entries を安全に呼び出すヘルパー（失敗時は current_dir を prev_dir に復帰）
pub fn try_get_entries(
    current_dir: &mut PathBuf,
    prev_dir: &Path,
    target: EntryType,
) -> io::Result<Vec<String>> {
    match get_entries(current_dir, target) {
        Ok(entries) => Ok(entries),
        Err(_) => {
            // アクセスできないため前のディレクトリに復帰
            *current_dir = prev_dir.to_path_buf();
            get_entries(current_dir, target)
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
        let dir = std::env::temp_dir().join(format!("cdx_fs_test_{}_{}", name, nanos));
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
