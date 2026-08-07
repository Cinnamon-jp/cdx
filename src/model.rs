// Copyright (c) 2026 Cinnamon-jp
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use crossterm::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    pub selected_border: String,
    #[serde(with = "color_serde")]
    pub selected_border_color: Color,
    #[serde(with = "color_serde")]
    pub path_foreground_color: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct SystemConfig {
    pub use_case_insensitive_search: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ui: UiConfig::default(),
            system: SystemConfig::default(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            selected_background_color: Color::Blue,
            selected_foreground_color: Color::White,
            selected_border: "none".to_string(),
            selected_border_color: Color::Black,
            path_foreground_color: Color::Cyan,
        }
    }
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            use_case_insensitive_search: true,
        }
    }
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
        // "reset" => Color::Reset,
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
        assert_eq!(parse_color_str("3b82f6"), None); // '#' が無いため None
        assert_eq!(
            parse_color_str("#f00"),
            Some(Color::Rgb { r: 255, g: 0, b: 0 })
        );
    }
}
