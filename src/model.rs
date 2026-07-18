// Copyright (c) 2026 Cinnamon-jp
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

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
    pub selected_background_color: String,
    pub selected_foreground_color: String,
    pub selected_border: String,
    pub selected_border_color: String,
    pub path_foreground_color: String,
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
            selected_background_color: "blue".to_string(),
            selected_foreground_color: "white".to_string(),
            selected_border: "none".to_string(),
            selected_border_color: "black".to_string(),
            path_foreground_color: "cyan".to_string(),
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
