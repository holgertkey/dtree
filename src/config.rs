use anyhow::{Context, Result};
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use crossterm::event::KeyCode;

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

    /// Show line numbers in fullscreen viewer by default
    #[serde(default = "default_show_line_numbers")]
    pub show_line_numbers: bool,

    /// Enable syntax highlighting for code files
    #[serde(default = "default_enable_syntax_highlighting")]
    pub enable_syntax_highlighting: bool,

    /// Syntax highlighting theme name
    #[serde(default = "default_syntax_theme")]
    pub syntax_theme: String,

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
            show_line_numbers: default_show_line_numbers(),
            enable_syntax_highlighting: default_enable_syntax_highlighting(),
            syntax_theme: default_syntax_theme(),
            colors: ThemeConfig::default(),
        }
    }
}

fn default_theme() -> String { "default".to_string() }
fn default_show_icons() -> bool { false }
fn default_split_position() -> u16 { 50 }
fn default_show_line_numbers() -> bool { false }
fn default_enable_syntax_highlighting() -> bool { true }
fn default_syntax_theme() -> String { "base16-ocean.dark".to_string() }

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

    /// External editor command for opening files
    #[serde(default = "default_editor")]
    pub editor: String,

    /// External file manager command
    #[serde(default = "default_file_manager")]
    pub file_manager: String,

    /// External hex editor for viewing binary files
    #[serde(default = "default_hex_editor")]
    pub hex_editor: String,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            max_file_lines: default_max_file_lines(),
            show_hidden: default_show_hidden(),
            follow_symlinks: default_follow_symlinks(),
            double_click_timeout_ms: default_double_click_timeout(),
            editor: default_editor(),
            file_manager: default_file_manager(),
            hex_editor: default_hex_editor(),
        }
    }
}

fn default_max_file_lines() -> usize { 1000 }
fn default_show_hidden() -> bool { false }
fn default_follow_symlinks() -> bool { false }
fn default_double_click_timeout() -> u64 { 500 }
fn default_editor() -> String { "nano".to_string() }
fn default_file_manager() -> String { "mc".to_string() }
fn default_hex_editor() -> String { "mcview".to_string() }

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

    /// Keys to open file in external editor
    #[serde(default = "default_open_editor_keys")]
    pub open_editor: Vec<String>,

    /// Keys to open in file manager
    #[serde(default = "default_open_file_manager_keys")]
    pub open_file_manager: Vec<String>,

    /// Keys to create bookmark
    #[serde(default = "default_create_bookmark_keys")]
    pub create_bookmark: Vec<String>,

    /// Keys to select bookmark
    #[serde(default = "default_select_bookmark_keys")]
    pub select_bookmark: Vec<String>,
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        Self {
            quit: default_quit_keys(),
            search: default_search_keys(),
            toggle_files: default_toggle_files_keys(),
            toggle_help: default_toggle_help_keys(),
            copy_path: default_copy_path_keys(),
            open_editor: default_open_editor_keys(),
            open_file_manager: default_open_file_manager_keys(),
            create_bookmark: default_create_bookmark_keys(),
            select_bookmark: default_select_bookmark_keys(),
        }
    }
}

fn default_quit_keys() -> Vec<String> { vec!["q".to_string(), "Esc".to_string()] }
fn default_search_keys() -> Vec<String> { vec!["/".to_string()] }
fn default_toggle_files_keys() -> Vec<String> { vec!["s".to_string()] }
fn default_toggle_help_keys() -> Vec<String> { vec!["i".to_string()] }
fn default_copy_path_keys() -> Vec<String> { vec!["c".to_string()] }
fn default_open_editor_keys() -> Vec<String> { vec!["e".to_string()] }
fn default_open_file_manager_keys() -> Vec<String> { vec!["o".to_string()] }
fn default_create_bookmark_keys() -> Vec<String> { vec!["m".to_string()] }
fn default_select_bookmark_keys() -> Vec<String> { vec!["'".to_string()] }

impl KeybindingsConfig {
    /// Check if a key matches any of the configured keys in the list
    fn matches_key(&self, key: KeyCode, configured_keys: &[String]) -> bool {
        let key_str = match key {
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Esc => "Esc".to_string(),
            KeyCode::Enter => "Enter".to_string(),
            KeyCode::Backspace => "Backspace".to_string(),
            KeyCode::Left => "Left".to_string(),
            KeyCode::Right => "Right".to_string(),
            KeyCode::Up => "Up".to_string(),
            KeyCode::Down => "Down".to_string(),
            KeyCode::Tab => "Tab".to_string(),
            KeyCode::Delete => "Delete".to_string(),
            KeyCode::Home => "Home".to_string(),
            KeyCode::End => "End".to_string(),
            KeyCode::PageUp => "PageUp".to_string(),
            KeyCode::PageDown => "PageDown".to_string(),
            _ => return false,
        };

        configured_keys.iter().any(|k| k.eq_ignore_ascii_case(&key_str))
    }

    pub fn is_quit(&self, key: KeyCode) -> bool {
        self.matches_key(key, &self.quit)
    }

    pub fn is_search(&self, key: KeyCode) -> bool {
        self.matches_key(key, &self.search)
    }

    pub fn is_toggle_files(&self, key: KeyCode) -> bool {
        self.matches_key(key, &self.toggle_files)
    }

    pub fn is_toggle_help(&self, key: KeyCode) -> bool {
        self.matches_key(key, &self.toggle_help)
    }

    pub fn is_copy_path(&self, key: KeyCode) -> bool {
        self.matches_key(key, &self.copy_path)
    }

    pub fn is_open_editor(&self, key: KeyCode) -> bool {
        self.matches_key(key, &self.open_editor)
    }

    pub fn is_open_file_manager(&self, key: KeyCode) -> bool {
        self.matches_key(key, &self.open_file_manager)
    }

    pub fn is_create_bookmark(&self, key: KeyCode) -> bool {
        self.matches_key(key, &self.create_bookmark)
    }

    pub fn is_select_bookmark(&self, key: KeyCode) -> bool {
        self.matches_key(key, &self.select_bookmark)
    }
}

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

    /// Load configuration with fallback order:
    /// 1. Global config (~/.config/dtree/config.toml)
    /// 2. Default config (if file is missing or has errors)
    ///
    /// If config file doesn't exist, it will be created automatically with default values.
    /// If config file has parse errors, detailed error is shown and default config is loaded.
    pub fn load() -> Self {
        let mut config = Config::default();

        // Get global config path
        if let Some(global_path) = Self::global_config_path() {
            // Create config file if it doesn't exist
            if !global_path.exists() {
                // Silently create default config file
                let _ = Self::create_default_file(&global_path);
            }

            // Load config from file
            if global_path.exists() {
                match Self::from_file(&global_path) {
                    Ok(global_config) => {
                        config = global_config;
                    }
                    Err(e) => {
                        // Show detailed error and exit
                        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                        eprintln!("⚠  Configuration file error!");
                        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                        eprintln!();
                        eprintln!("Config file: {}", global_path.display());
                        eprintln!();
                        eprintln!("Error details:");
                        eprintln!("{:#}", e);
                        eprintln!();
                        eprintln!("To fix:");
                        eprintln!("  1. Edit the config file and fix the syntax error");
                        eprintln!("  2. Or delete the file - it will be recreated with defaults");
                        eprintln!();
                        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                        std::process::exit(1);
                    }
                }
            }
        }

        config
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

# Show line numbers in fullscreen viewer by default (toggle with 'n' key)
show_line_numbers = false

# Enable syntax highlighting for code files
enable_syntax_highlighting = true

# Syntax highlighting theme
# Available themes: "base16-ocean.dark", "base16-ocean.light", "InspiredGitHub",
#                   "Solarized (dark)", "Solarized (light)", "Monokai Extended"
syntax_theme = "base16-ocean.dark"

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

# External editor for opening files (press 'e' to open)
# Default: nano (if not installed, change to your preferred editor)
# Popular options:
#   - Terminal editors: "nano", "vim", "nvim", "emacs", "micro", "helix"
#   - GUI editors (if terminal wrapper available): "code", "subl", "gedit"
editor = "nano"

# External file manager (press 'o' to open)
# Default: mc (Midnight Commander)
# Popular terminal file managers:
#   - "mc"      - Midnight Commander (classic two-panel interface)
#   - "ranger"  - Vi-like file manager with image preview support
#   - "nnn"     - Fast and minimal file manager
#   - "lf"      - Terminal file manager inspired by ranger
#   - "vifm"    - Vi-like file manager with two panels
#   - "broot"   - Navigate directories with fuzzy search
#   - "yazi"    - Modern terminal file manager
file_manager = "mc"

# External hex editor for binary files (press 'e' on binary file in fullscreen mode)
# Default: mcview (part of Midnight Commander)
# Popular hex viewers:
#   - "mcview"  - Midnight Commander's internal viewer (recommended)
#   - "hexyl"   - Modern, colorful hex viewer (install: cargo install hexyl)
#   - "xxd"     - Standard hex dump utility (included with vim)
#   - "hexdump" - Classic hex dump tool
#   - "hd"      - Alias for hexdump -C
hex_editor = "mcview"

[keybindings]
# Key bindings (each can have multiple keys)
quit = ["q", "Esc"]
search = ["/"]
toggle_files = ["s"]
toggle_help = ["i"]
copy_path = ["c"]
open_editor = ["e"]
open_file_manager = ["o"]
create_bookmark = ["m"]
select_bookmark = ["'"]
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
