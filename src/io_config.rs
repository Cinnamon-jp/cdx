// Copyright (c) 2026 Cinnamon-jp
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use directories::ProjectDirs;
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::model::Config;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// proj_path にはツール名を入れる
pub fn get_config_toml_or_default(proj_path: PathBuf) -> Result<Config> {
    let proj_str = proj_path
        .to_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid project path"))?;
    let proj_dirs = ProjectDirs::from("", "", proj_str).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Could not determine project directories",
        )
    })?;
    let config_dir = proj_dirs.config_dir();
    let config_path = config_dir.join("config.toml");

    if !config_path.exists() {
        return Ok(Config::default());
    }

    parse_toml(config_path)
}

fn parse_toml(path: PathBuf) -> Result<Config> {
    let content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    // 競合しない一時ファイルのパスを作成する関数
    fn temp_file_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("cdx_test_{}_{}.toml", name, nanos))
    }

    #[test]
    fn test_parse_toml_valid() -> Result<()> {
        use crossterm::style::Color;

        let path = temp_file_path("valid");
        let toml_content = r#"
[ui]
selected_background_color = "red"
selected_foreground_color = "black"
selected_border = "line"
path_foreground_color = "magenta"

[system]
use_case_insensitive_search = false
"#;
        fs::write(&path, toml_content)?;

        let config = parse_toml(path.clone())?;
        let _ = fs::remove_file(path);

        assert_eq!(config.ui.selected_background_color, Color::Red);
        assert_eq!(config.ui.selected_foreground_color, Color::Black);
        assert_eq!(config.ui.selected_border, "line");
        assert_eq!(config.ui.path_foreground_color, Color::Magenta);
        assert!(!config.system.use_case_insensitive_search);

        Ok(())
    }

    #[test]
    fn test_parse_toml_hex_color() -> Result<()> {
        use crossterm::style::Color;

        let path = temp_file_path("hex_color");
        let toml_content = r##"
[ui]
selected_background_color = "#ff0000"
selected_foreground_color = "#00ff00"
"##;
        fs::write(&path, toml_content)?;

        let config = parse_toml(path.clone())?;
        let _ = fs::remove_file(path);

        assert_eq!(
            config.ui.selected_background_color,
            Color::Rgb { r: 255, g: 0, b: 0 }
        );
        assert_eq!(
            config.ui.selected_foreground_color,
            Color::Rgb { r: 0, g: 255, b: 0 }
        );

        Ok(())
    }

    #[test]
    fn test_parse_toml_defaults() -> Result<()> {
        let path = temp_file_path("defaults");
        let toml_content = r#""#;
        fs::write(&path, toml_content)?;

        let config = parse_toml(path.clone())?;
        let _ = fs::remove_file(path);

        let default_config = Config::default();
        assert_eq!(config, default_config);

        Ok(())
    }
}
