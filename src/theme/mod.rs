use ratatui::style::Color;
use serde::{Deserialize, Serialize};

pub mod presets;

/// Theme configuration with customizable colors
/// All fields are Option<String>:
/// - None means the color is not set (use preset theme value)
/// - Some(value) means the color is explicitly set (override preset)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Color for selected items
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_color: Option<String>,

    /// Color for directory names
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub directory_color: Option<String>,

    /// Color for file names
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_color: Option<String>,

    /// Color for borders
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_color: Option<String>,

    /// Color for error messages
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_color: Option<String>,

    /// Color for search highlights (directory search)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highlight_color: Option<String>,

    /// Color for file content search highlights
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_search_highlight_color: Option<String>,

    /// Color for cursor/selection highlight (search & bookmarks)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor_color: Option<String>,

    /// Color for tree cursor/selection highlight
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tree_cursor_color: Option<String>,

    /// Background color for tree cursor/selection line
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tree_cursor_bg_color: Option<String>,

    /// Color for main window border
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub main_border_color: Option<String>,

    /// Color for panel borders (search, bookmarks)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub panel_border_color: Option<String>,

    /// Color for background (optional, uses terminal default if not set)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            selected_color: None,
            directory_color: None,
            file_color: None,
            border_color: None,
            error_color: None,
            highlight_color: None,
            file_search_highlight_color: None,
            cursor_color: None,
            tree_cursor_color: None,
            tree_cursor_bg_color: None,
            main_border_color: None,
            panel_border_color: None,
            background_color: None,
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

    /// Get fallback color values (used when no preset is set and no custom color is provided)
    pub fn fallback_colors() -> Self {
        Self {
            selected_color: Some("cyan".to_string()),
            directory_color: Some("blue".to_string()),
            file_color: Some("white".to_string()),
            border_color: Some("gray".to_string()),
            error_color: Some("red".to_string()),
            highlight_color: Some("yellow".to_string()),
            file_search_highlight_color: Some("yellow".to_string()),
            cursor_color: Some("yellow".to_string()),
            tree_cursor_color: Some("dim".to_string()),
            tree_cursor_bg_color: Some("dim".to_string()),
            main_border_color: Some("gray".to_string()),
            panel_border_color: Some("cyan".to_string()),
            background_color: Some("reset".to_string()),
        }
    }
}
