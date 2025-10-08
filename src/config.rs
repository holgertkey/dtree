use anyhow::{Context, Result};
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Theme configuration with customizable colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Color for selected items
    #[serde(default = "default_selected_color")]
    pub selected_color: String,

    /// Color for directory names
    #[serde(default = "default_directory_color")]
    pub directory_color: String,

    /// Color for file names
    #[serde(default = "default_file_color")]
    pub file_color: String,

    /// Color for borders
    #[serde(default = "default_border_color")]
    pub border_color: String,

    /// Color for error messages
    #[serde(default = "default_error_color")]
    pub error_color: String,

    /// Color for search highlights
    #[serde(default = "default_highlight_color")]
    pub highlight_color: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            selected_color: default_selected_color(),
            directory_color: default_directory_color(),
            file_color: default_file_color(),
            border_color: default_border_color(),
            error_color: default_error_color(),
            highlight_color: default_highlight_color(),
        }
    }
}

impl ThemeConfig {
    /// Parse a color string to ratatui Color
    pub fn parse_color(color_str: &str) -> Color {
        match color_str.to_lowercase().as_str() {
            "black" => Color::Black,
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            "gray" | "grey" => Color::Gray,
            "darkgray" | "darkgrey" => Color::DarkGray,
            "lightred" => Color::LightRed,
            "lightgreen" => Color::LightGreen,
            "lightyellow" => Color::LightYellow,
            "lightblue" => Color::LightBlue,
            "lightmagenta" => Color::LightMagenta,
            "lightcyan" => Color::LightCyan,
            "white" => Color::White,
            // Try to parse as RGB hex color (#RRGGBB)
            s if s.starts_with('#') && s.len() == 7 => {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    u8::from_str_radix(&s[1..3], 16),
                    u8::from_str_radix(&s[3..5], 16),
                    u8::from_str_radix(&s[5..7], 16),
                ) {
                    Color::Rgb(r, g, b)
                } else {
                    Color::White // Fallback
                }
            }
            // Try to parse as indexed color (0-255)
            s => {
                if let Ok(idx) = s.parse::<u8>() {
                    Color::Indexed(idx)
                } else {
                    Color::White // Fallback
                }
            }
        }
    }
}

// Default color functions
fn default_selected_color() -> String { "cyan".to_string() }
fn default_directory_color() -> String { "blue".to_string() }
fn default_file_color() -> String { "white".to_string() }
fn default_border_color() -> String { "gray".to_string() }
fn default_error_color() -> String { "red".to_string() }
fn default_highlight_color() -> String { "yellow".to_string() }

/// Appearance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    /// Theme name (can be expanded later for preset themes)
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Show file type icons (requires nerd fonts)
    #[serde(default = "default_show_icons")]
    pub show_icons: bool,

    /// Split position percentage (20-80)
    #[serde(default = "default_split_position")]
    pub split_position: u16,

    /// Custom theme colors
    #[serde(default)]
    pub colors: ThemeConfig,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            show_icons: default_show_icons(),
            split_position: default_split_position(),
            colors: ThemeConfig::default(),
        }
    }
}

fn default_theme() -> String { "default".to_string() }
fn default_show_icons() -> bool { false }
fn default_split_position() -> u16 { 50 }

/// Behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    /// Maximum number of lines to read from files
    #[serde(default = "default_max_file_lines")]
    pub max_file_lines: usize,

    /// Show hidden files (dotfiles)
    #[serde(default = "default_show_hidden")]
    pub show_hidden: bool,

    /// Follow symbolic links
    #[serde(default = "default_follow_symlinks")]
    pub follow_symlinks: bool,

    /// Double-click timeout in milliseconds
    #[serde(default = "default_double_click_timeout")]
    pub double_click_timeout_ms: u64,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            max_file_lines: default_max_file_lines(),
            show_hidden: default_show_hidden(),
            follow_symlinks: default_follow_symlinks(),
            double_click_timeout_ms: default_double_click_timeout(),
        }
    }
}

fn default_max_file_lines() -> usize { 1000 }
fn default_show_hidden() -> bool { false }
fn default_follow_symlinks() -> bool { false }
fn default_double_click_timeout() -> u64 { 500 }

/// Keybindings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingsConfig {
    /// Keys to quit the application
    #[serde(default = "default_quit_keys")]
    pub quit: Vec<String>,

    /// Keys to toggle search mode
    #[serde(default = "default_search_keys")]
    pub search: Vec<String>,

    /// Keys to toggle file viewer
    #[serde(default = "default_toggle_files_keys")]
    pub toggle_files: Vec<String>,

    /// Keys to toggle help screen
    #[serde(default = "default_toggle_help_keys")]
    pub toggle_help: Vec<String>,

    /// Keys to copy path to clipboard
    #[serde(default = "default_copy_path_keys")]
    pub copy_path: Vec<String>,
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        Self {
            quit: default_quit_keys(),
            search: default_search_keys(),
            toggle_files: default_toggle_files_keys(),
            toggle_help: default_toggle_help_keys(),
            copy_path: default_copy_path_keys(),
        }
    }
}

fn default_quit_keys() -> Vec<String> { vec!["q".to_string(), "Esc".to_string()] }
fn default_search_keys() -> Vec<String> { vec!["/".to_string()] }
fn default_toggle_files_keys() -> Vec<String> { vec!["s".to_string()] }
fn default_toggle_help_keys() -> Vec<String> { vec!["i".to_string()] }
fn default_copy_path_keys() -> Vec<String> { vec!["c".to_string()] }

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub appearance: AppearanceConfig,

    #[serde(default)]
    pub behavior: BehaviorConfig,

    #[serde(default)]
    pub keybindings: KeybindingsConfig,
}

impl Config {
    /// Parse a color string to ratatui Color
    pub fn parse_color(color_str: &str) -> Color {
        ThemeConfig::parse_color(color_str)
    }

    /// Load configuration from a file
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        Ok(config)
    }

    /// Get the global config file path (~/.config/dtree/config.toml)
    pub fn global_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("dtree").join("config.toml"))
    }

    /// Get the local config file path (.dtree.toml in current directory)
    pub fn local_config_path(dir: &Path) -> PathBuf {
        dir.join(".dtree.toml")
    }

    /// Load configuration with fallback order:
    /// 1. Local config (.dtree.toml in the directory)
    /// 2. Global config (~/.config/dtree/config.toml)
    /// 3. Default config
    pub fn load(dir: &Path) -> Self {
        let mut config = Config::default();

        // Try to load global config
        if let Some(global_path) = Self::global_config_path() {
            if global_path.exists() {
                if let Ok(global_config) = Self::from_file(&global_path) {
                    config = config.merge(global_config);
                }
            }
        }

        // Try to load local config (overrides global)
        let local_path = Self::local_config_path(dir);
        if local_path.exists() {
            if let Ok(local_config) = Self::from_file(&local_path) {
                config = config.merge(local_config);
            }
        }

        config
    }

    /// Merge another config into this one (other takes precedence)
    pub fn merge(self, other: Self) -> Self {
        Self {
            appearance: merge_appearance(self.appearance, other.appearance),
            behavior: merge_behavior(self.behavior, other.behavior),
            keybindings: merge_keybindings(self.keybindings, other.keybindings),
        }
    }

    /// Create a default config file with comments
    pub fn create_default_file(path: &Path) -> Result<()> {
        let default_config = r#"# dtree configuration file
# This file uses TOML format: https://toml.io

[appearance]
# Theme name (currently only "default" is supported)
theme = "default"

# Show file type icons (requires nerd fonts)
show_icons = false

# Split position for file viewer (20-80, percentage)
split_position = 50

# Custom theme colors
[appearance.colors]
# Color names: black, red, green, yellow, blue, magenta, cyan, gray, white
# Or RGB hex: #RRGGBB
# Or indexed: 0-255
selected_color = "cyan"
directory_color = "blue"
file_color = "white"
border_color = "gray"
error_color = "red"
highlight_color = "yellow"

[behavior]
# Maximum number of lines to read from files
max_file_lines = 1000

# Show hidden files (dotfiles)
show_hidden = false

# Follow symbolic links
follow_symlinks = false

# Double-click timeout in milliseconds
double_click_timeout_ms = 500

[keybindings]
# Key bindings (each can have multiple keys)
quit = ["q", "Esc"]
search = ["/"]
toggle_files = ["s"]
toggle_help = ["i"]
copy_path = ["c"]
"#;

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
        }

        fs::write(path, default_config)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;

        Ok(())
    }
}

// Helper functions for merging configs
fn merge_appearance(_base: AppearanceConfig, other: AppearanceConfig) -> AppearanceConfig {
    AppearanceConfig {
        theme: other.theme,
        show_icons: other.show_icons,
        split_position: other.split_position,
        colors: other.colors,
    }
}

fn merge_behavior(_base: BehaviorConfig, other: BehaviorConfig) -> BehaviorConfig {
    BehaviorConfig {
        max_file_lines: other.max_file_lines,
        show_hidden: other.show_hidden,
        follow_symlinks: other.follow_symlinks,
        double_click_timeout_ms: other.double_click_timeout_ms,
    }
}

fn merge_keybindings(_base: KeybindingsConfig, other: KeybindingsConfig) -> KeybindingsConfig {
    KeybindingsConfig {
        quit: other.quit,
        search: other.search,
        toggle_files: other.toggle_files,
        toggle_help: other.toggle_help,
        copy_path: other.copy_path,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.appearance.split_position, 50);
        assert_eq!(config.behavior.max_file_lines, 1000);
        assert!(!config.behavior.show_hidden);
    }

    #[test]
    fn test_color_parsing() {
        assert!(matches!(ThemeConfig::parse_color("red"), Color::Red));
        assert!(matches!(ThemeConfig::parse_color("blue"), Color::Blue));
        assert!(matches!(ThemeConfig::parse_color("#FF0000"), Color::Rgb(255, 0, 0)));
    }
}
