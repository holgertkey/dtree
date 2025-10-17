use ratatui::style::Color;
use serde::{Deserialize, Serialize};

pub mod presets;

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

    /// Color for cursor/selection highlight (search & bookmarks)
    #[serde(default = "default_cursor_color")]
    pub cursor_color: String,

    /// Color for tree cursor/selection highlight
    #[serde(default = "default_tree_cursor_color")]
    pub tree_cursor_color: String,

    /// Color for main window border
    #[serde(default = "default_main_border_color")]
    pub main_border_color: String,

    /// Color for panel borders (search, bookmarks)
    #[serde(default = "default_panel_border_color")]
    pub panel_border_color: String,

    /// Color for background (optional, uses terminal default if not set)
    #[serde(default = "default_background_color")]
    pub background_color: String,
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
            cursor_color: default_cursor_color(),
            tree_cursor_color: default_tree_cursor_color(),
            main_border_color: default_main_border_color(),
            panel_border_color: default_panel_border_color(),
            background_color: default_background_color(),
        }
    }
}

impl ThemeConfig {
    /// Parse a color string to ratatui Color
    pub fn parse_color(color_str: &str) -> Color {
        match color_str.to_lowercase().as_str() {
            "reset" => Color::Reset, // Use terminal default
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
                    Color::Reset // Fallback to terminal default
                }
            }
            // Try to parse as indexed color (0-255)
            s => {
                if let Ok(idx) = s.parse::<u8>() {
                    Color::Indexed(idx)
                } else {
                    Color::Reset // Fallback to terminal default
                }
            }
        }
    }

    /// Get preset theme colors by theme name
    pub fn get_preset_theme(theme_name: &str) -> Option<Self> {
        presets::get_preset(theme_name)
    }
}

// Default color functions
fn default_selected_color() -> String { "cyan".to_string() }
fn default_directory_color() -> String { "blue".to_string() }
fn default_file_color() -> String { "white".to_string() }
fn default_border_color() -> String { "gray".to_string() }
fn default_error_color() -> String { "red".to_string() }
fn default_highlight_color() -> String { "yellow".to_string() }
fn default_cursor_color() -> String { "yellow".to_string() }
fn default_tree_cursor_color() -> String { "dim".to_string() }
fn default_main_border_color() -> String { "gray".to_string() }
fn default_panel_border_color() -> String { "cyan".to_string() }
fn default_background_color() -> String { "reset".to_string() }
