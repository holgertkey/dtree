use anyhow::{Context, Result};
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use crossterm::event::KeyCode;

use crate::theme::ThemeConfig;

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
fn default_split_position() -> u16 { 20 }
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

fn default_max_file_lines() -> usize { 10000 }
fn default_show_hidden() -> bool { true }
fn default_follow_symlinks() -> bool { true }
fn default_double_click_timeout() -> u64 { 500 }
fn default_editor() -> String { "nvim".to_string() }
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

    /// Keys to toggle line numbers in fullscreen viewer
    #[serde(default = "default_show_line_numbers_keys")]
    pub show_line_numbers: Vec<String>,
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
            show_line_numbers: default_show_line_numbers_keys(),
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
fn default_show_line_numbers_keys() -> Vec<String> { vec!["l".to_string()] }

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

    pub fn is_show_line_numbers(&self, key: KeyCode) -> bool {
        self.matches_key(key, &self.show_line_numbers)
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

    /// Get a color value (guaranteed to be Some after load())
    pub fn get_color(opt: &Option<String>) -> &str {
        opt.as_ref().expect("Color should be resolved after config load")
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

        // Apply color resolution:
        // 1. Use explicitly set color from config file (if Some)
        // 2. Otherwise, use preset theme color (if theme is set and preset has color)
        // 3. Otherwise, use fallback default color
        let preset = ThemeConfig::get_preset_theme(&config.appearance.theme);
        let fallback = ThemeConfig::fallback_colors();

        // Helper macro to apply color resolution
        macro_rules! resolve_color {
            ($field:ident) => {
                config.appearance.colors.$field = config.appearance.colors.$field
                    .or_else(|| preset.as_ref().and_then(|p| p.$field.clone()))
                    .or_else(|| fallback.$field.clone());
            };
        }

        resolve_color!(selected_color);
        resolve_color!(directory_color);
        resolve_color!(file_color);
        resolve_color!(border_color);
        resolve_color!(error_color);
        resolve_color!(highlight_color);
        resolve_color!(file_search_highlight_color);
        resolve_color!(cursor_color);
        resolve_color!(tree_cursor_color);
        resolve_color!(tree_cursor_bg_color);
        resolve_color!(main_border_color);
        resolve_color!(panel_border_color);
        resolve_color!(background_color);

        config
    }

    /// Create a default config file with comments
    pub fn create_default_file(path: &Path) -> Result<()> {
        let default_config = r#"# dtree configuration file
# This file uses TOML format: https://toml.io

[appearance]
# Theme name - preset color schemes
# Available themes:
#   "default"    - Classic terminal colors (blue dirs, cyan selection)
#   "gruvbox"    - Warm, high contrast theme inspired by Gruvbox
#   "nord"       - Cold, muted colors inspired by Nord theme
#   "tokyonight" - Modern dark theme with vibrant colors
#   "dracula"    - Popular dark theme with high contrast
#
# You can override individual colors in [appearance.colors] section below
# Preset themes provide a good starting point with harmonious color palettes
theme = "default"

# Show file type icons (requires nerd fonts)
show_icons = false

# Split position for file viewer (20-80, percentage)
split_position = 20

# Show line numbers in fullscreen viewer by default (toggle with 'l' key)
show_line_numbers = false

# Enable syntax highlighting for code files
enable_syntax_highlighting = true

# Syntax highlighting theme
# Available themes: "base16-ocean.dark", "base16-ocean.light", "InspiredGitHub",
#                   "Solarized (dark)", "Solarized (light)", "Monokai Extended"
syntax_theme = "base16-ocean.dark"

# Custom theme colors
# These colors override the preset theme colors above
# By default, all colors are commented out to use the preset theme
# Uncomment and modify any color to override the theme
[appearance.colors]
# Color formats:
#   - Color names: black, red, green, yellow, blue, magenta, cyan, gray, white
#   - RGB hex: #RRGGBB (e.g., #fe8019)
#   - Indexed: 0-255 (256-color palette)
#   - "reset" - use terminal default color
#
# selected_color = "cyan"           # Color for selected item text
# directory_color = "blue"          # Color for directory names
# file_color = "white"              # Color for file names
# border_color = "gray"             # Color for UI borders
# error_color = "red"               # Color for error messages
# highlight_color = "yellow"        # Color for fuzzy search character highlighting (directory search)
# file_search_highlight_color = "yellow"  # Color for file content search highlighting
# cursor_color = "yellow"           # Cursor highlight for search & bookmarks
# tree_cursor_color = "dim"         # Cursor highlight for tree ("dim" = no color, just dimming)
# tree_cursor_bg_color = "dim"      # Cursor background for tree ("dim" = no background color)
# main_border_color = "gray"        # Main window border color
# panel_border_color = "cyan"       # Panel borders (search, bookmarks)
# background_color = "reset"        # Background color ("reset" = terminal default)

[behavior]
# Maximum number of lines to read from files
max_file_lines = 10000

# Show hidden files (dotfiles)
show_hidden = true

# Follow symbolic links
follow_symlinks = true

# Double-click timeout in milliseconds
double_click_timeout_ms = 500

# External editor for opening files (press 'e' to open)
# Default: nvim (if not installed, change to your preferred editor)
# Popular options:
#   - Terminal editors: "nvim", "vim", "nano", "emacs", "micro", "helix"
#   - GUI editors (if terminal wrapper available): "code", "subl", "gedit"
editor = "nvim"

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
show_line_numbers = ["l"]
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
        assert_eq!(config.appearance.split_position, 20);
        assert_eq!(config.behavior.max_file_lines, 10000);
        assert!(config.behavior.show_hidden);
    }

    #[test]
    fn test_color_parsing() {
        assert!(matches!(ThemeConfig::parse_color("red"), Color::Red));
        assert!(matches!(ThemeConfig::parse_color("blue"), Color::Blue));
        assert!(matches!(ThemeConfig::parse_color("#FF0000"), Color::Rgb(255, 0, 0)));
    }
}
