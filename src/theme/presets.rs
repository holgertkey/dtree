use super::ThemeConfig;

/// Get preset theme by name
pub fn get_preset(theme_name: &str) -> Option<ThemeConfig> {
    match theme_name.to_lowercase().as_str() {
        "default" => Some(default_theme()),
        "gruvbox" => Some(gruvbox_theme()),
        "nord" => Some(nord_theme()),
        _ => None,
    }
}

/// Default theme - Classic terminal colors
fn default_theme() -> ThemeConfig {
    ThemeConfig {
        selected_color: Some("cyan".to_string()),
        directory_color: Some("blue".to_string()),
        file_color: Some("white".to_string()),
        border_color: Some("gray".to_string()),
        error_color: Some("red".to_string()),
        highlight_color: Some("yellow".to_string()),
        cursor_color: Some("yellow".to_string()),     // for search & bookmarks
        tree_cursor_color: Some("dim".to_string()),   // "dim" = no color, just dimming
        tree_cursor_bg_color: Some("dim".to_string()), // "dim" = no background color
        main_border_color: Some("gray".to_string()),  // main window border
        panel_border_color: Some("cyan".to_string()), // panel borders (search, bookmarks)
        background_color: Some("reset".to_string()),  // terminal default
    }
}

/// Gruvbox theme - Warm, high contrast theme inspired by Gruvbox
fn gruvbox_theme() -> ThemeConfig {
    ThemeConfig {
        selected_color: Some("#fe8019".to_string()),  // bright orange
        directory_color: Some("#83a598".to_string()), // bright blue
        file_color: Some("#ebdbb2".to_string()),      // light foreground
        border_color: Some("#928374".to_string()),    // gray
        error_color: Some("#fb4934".to_string()),     // bright red
        highlight_color: Some("#fabd2f".to_string()), // bright yellow
        cursor_color: Some("#fabd2f".to_string()),    // yellow for search & bookmarks
        tree_cursor_color: Some("dim".to_string()),   // "dim" = no color, just dimming
        tree_cursor_bg_color: Some("dim".to_string()), // "dim" = no background color
        main_border_color: Some("#928374".to_string()), // gray border
        panel_border_color: Some("#fe8019".to_string()), // orange panel borders (search, bookmarks)
        background_color: Some("#282828".to_string()), // gruvbox dark bg
    }
}

/// Nord theme - Cold, muted colors inspired by Nord theme
fn nord_theme() -> ThemeConfig {
    ThemeConfig {
        selected_color: Some("#88c0d0".to_string()), // frost cyan
        directory_color: Some("#81a1c1".to_string()), // frost blue
        file_color: Some("#eceff4".to_string()),     // snow white
        border_color: Some("#4c566a".to_string()),   // polar night gray
        error_color: Some("#bf616a".to_string()),    // aurora red
        highlight_color: Some("#ebcb8b".to_string()), // aurora yellow
        cursor_color: Some("#ebcb8b".to_string()),   // yellow for search & bookmarks
        tree_cursor_color: Some("dim".to_string()),  // "dim" = no color, just dimming
        tree_cursor_bg_color: Some("dim".to_string()), // "dim" = no background color
        main_border_color: Some("#4c566a".to_string()), // polar night gray border
        panel_border_color: Some("#88c0d0".to_string()), // cyan panel borders (search, bookmarks)
        background_color: Some("#2e3440".to_string()), // nord dark bg
    }
}
