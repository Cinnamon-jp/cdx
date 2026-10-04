// Copyright (c) 2026 Cinnamon-jp
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use crossterm::style::Color;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Config {
    pub ui: UiConfig,
    pub system: SystemConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct UiConfig {
    #[serde(with = "color_serde")]
    pub selected_background_color: Color,
    #[serde(with = "color_serde")]
    pub selected_foreground_color: Color,
    #[serde(with = "color_serde")]
    pub path_foreground_color: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct SystemConfig {
    pub case_insensitive_search: bool,
    pub partial_navigation_fallback: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            selected_background_color: Color::DarkGrey,
            selected_foreground_color: Color::Cyan,
            path_foreground_color: Color::Cyan,
        }
    }
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            case_insensitive_search: true,
            partial_navigation_fallback: false,
        }
    }
}

/// 設定ファイル (~/.config/cdx/config.toml) を読み込み、存在しない場合やエラー時はデフォルト設定を返す
pub fn load_config_or_default() -> Config {
    get_config_path()
        .and_then(|path| parse_toml(&path).ok())
        .unwrap_or_default()
}

/// 設定ファイルのパスを取得
pub fn get_config_path() -> Option<PathBuf> {
    let proj_dirs = ProjectDirs::from("", "", "cdx")?;
    Some(proj_dirs.config_dir().join("config.toml"))
}

/// 指定したパスのTOMLをパース
pub fn parse_toml(path: &PathBuf) -> Result<Config> {
    let content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

pub fn parse_color_str(s: &str) -> Option<Color> {
    let s_clean = s.trim().to_lowercase();
    match s_clean.as_str() {
        "black" => Some(Color::Black),
        "gray" | "dark white" => Some(Color::Grey), // アメリカ英語の gray を採用
        "white" => Some(Color::White),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "dark gray" => Some(Color::DarkGrey), // アメリカ英語の gray を採用
        "dark red" => Some(Color::DarkRed),
        "dark green" => Some(Color::DarkGreen),
        "dark yellow" => Some(Color::DarkYellow),
        "dark blue" => Some(Color::DarkBlue),
        "dark magenta" => Some(Color::DarkMagenta),
        "dark cyan" => Some(Color::DarkCyan),
        _ => parse_hex_color(&s_clean),
    }
}

pub fn parse_hex_color(s: &str) -> Option<Color> {
    let hex = s.trim().strip_prefix('#')?;
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color::Rgb { r, g, b })
        }
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            Some(Color::Rgb { r, g, b })
        }
        _ => None,
    }
}

pub mod color_serde {
    use crossterm::style::Color;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match color {
            Color::Reset => "reset".to_string(),
            Color::Black => "black".to_string(),
            Color::DarkRed => "darkred".to_string(),
            Color::DarkGreen => "darkgreen".to_string(),
            Color::DarkYellow => "darkyellow".to_string(),
            Color::DarkBlue => "darkblue".to_string(),
            Color::DarkMagenta => "darkmagenta".to_string(),
            Color::DarkCyan => "darkcyan".to_string(),
            Color::Grey => "grey".to_string(),
            Color::DarkGrey => "darkgrey".to_string(),
            Color::Red => "red".to_string(),
            Color::Green => "green".to_string(),
            Color::Yellow => "yellow".to_string(),
            Color::Blue => "blue".to_string(),
            Color::Magenta => "magenta".to_string(),
            Color::Cyan => "cyan".to_string(),
            Color::White => "white".to_string(),
            Color::Rgb { r, g, b } => format!("#{:02x}{:02x}{:02x}", r, g, b),
            Color::AnsiValue(val) => format!("ansi({})", val),
        };
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(super::parse_color_str(&s).unwrap_or(Color::Reset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    // 競合しない一時ファイルのパスを作成
    fn temp_file_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("cdx_test_{}_{}.toml", name, nanos))
    }

    #[test]
    fn test_parse_color_str_named() {
        assert_eq!(parse_color_str("red"), Some(Color::Red));
        assert_eq!(parse_color_str("BLUE"), Some(Color::Blue));
        assert_eq!(parse_color_str("cyan"), Some(Color::Cyan));
        assert_eq!(parse_color_str("unknown"), None);
    }

    #[test]
    fn test_parse_color_str_hex() {
        assert_eq!(
            parse_color_str("#ff0000"),
            Some(Color::Rgb { r: 255, g: 0, b: 0 })
        );
        assert_eq!(parse_color_str("3b82f6"), None); // '#' がないため None
        assert_eq!(
            parse_color_str("#f00"),
            Some(Color::Rgb { r: 255, g: 0, b: 0 })
        );
    }

    #[test]
    fn test_parse_toml_valid() -> Result<()> {
        let path = temp_file_path("valid");
        let toml_content = r#"
[ui]
selected_background_color = "red"
selected_foreground_color = "black"
path_foreground_color = "magenta"

[system]
case_insensitive_search = false
partial_navigation_fallback = false
"#;
        fs::write(&path, toml_content)?;

        let config = parse_toml(&path)?;
        let _ = fs::remove_file(path);

        assert_eq!(config.ui.selected_background_color, Color::Red);
        assert_eq!(config.ui.selected_foreground_color, Color::Black);
        assert_eq!(config.ui.path_foreground_color, Color::Magenta);
        assert!(!config.system.case_insensitive_search);
        assert!(!config.system.partial_navigation_fallback);

        Ok(())
    }

    #[test]
    fn test_parse_toml_hex_color() -> Result<()> {
        let path = temp_file_path("hex_color");
        let toml_content = r##"
[ui]
selected_background_color = "#ff0000"
selected_foreground_color = "#00ff00"
"##;
        fs::write(&path, toml_content)?;

        let config = parse_toml(&path)?;
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

        let config = parse_toml(&path)?;
        let _ = fs::remove_file(path);

        let default_config = Config::default();
        assert_eq!(config, default_config);

        Ok(())
    }

    #[test]
    fn test_parse_toml_partial() -> Result<()> {
        let path = temp_file_path("partial");
        let toml_content = r#"
[ui]
selected_background_color = "blue"

[system]
partial_navigation_fallback = true
"#;
        fs::write(&path, toml_content)?;

        let config = parse_toml(&path)?;
        let _ = fs::remove_file(path);

        // 指定した値が反映されていること
        assert_eq!(config.ui.selected_background_color, Color::Blue);
        assert!(config.system.partial_navigation_fallback);

        // 未指定の項目に正しくデフォルト値が割り当てられていること
        assert_eq!(config.ui.selected_foreground_color, Color::Cyan);
        assert_eq!(config.ui.path_foreground_color, Color::Cyan);
        assert!(config.system.case_insensitive_search);

        Ok(())
    }

    #[test]
    fn test_parse_toml_only_ui_or_system() -> Result<()> {
        // [ui] のみ指定、[system] 省略
        let path1 = temp_file_path("only_ui");
        fs::write(&path1, "[ui]\nselected_background_color = \"red\"\n")?;
        let config1 = parse_toml(&path1)?;
        let _ = fs::remove_file(path1);

        assert_eq!(config1.ui.selected_background_color, Color::Red);
        assert_eq!(config1.ui.selected_foreground_color, Color::Cyan);
        assert_eq!(config1.system, SystemConfig::default());

        // [system] のみ指定、[ui] 省略
        let path2 = temp_file_path("only_system");
        fs::write(&path2, "[system]\ncase_insensitive_search = false\n")?;
        let config2 = parse_toml(&path2)?;
        let _ = fs::remove_file(path2);

        assert_eq!(config2.ui, UiConfig::default());
        assert!(!config2.system.case_insensitive_search);
        assert!(!config2.system.partial_navigation_fallback);

        Ok(())
    }
}
