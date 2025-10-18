use super::ThemeConfig;

/// Get preset theme by name
pub fn get_preset(theme_name: &str) -> Option<ThemeConfig> {
    match theme_name.to_lowercase().as_str() {
        "default" => Some(default_theme()),
        "gruvbox" => Some(gruvbox_theme()),
        "nord" => Some(nord_theme()),
        "tokyonight" => Some(tokyonight_theme()),
        "dracula" => Some(dracula_theme()),
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

/// Tokyo Night theme - Modern dark theme with vibrant colors
fn tokyonight_theme() -> ThemeConfig {
    ThemeConfig {
        selected_color: Some("#7aa2f7".to_string()),  // blue
        directory_color: Some("#7dcfff".to_string()), // cyan
        file_color: Some("#a9b1d6".to_string()),      // light gray-blue
        border_color: Some("#3b4261".to_string()),    // dark gray
        error_color: Some("#f7768e".to_string()),     // red
        highlight_color: Some("#e0af68".to_string()), // yellow
        cursor_color: Some("#bb9af7".to_string()),    // purple for search & bookmarks
        tree_cursor_color: Some("dim".to_string()),   // "dim" = no color, just dimming
        tree_cursor_bg_color: Some("dim".to_string()), // "dim" = no background color
        main_border_color: Some("#3b4261".to_string()), // dark gray border
        panel_border_color: Some("#9d7cd8".to_string()), // purple panel borders (search, bookmarks)
        background_color: Some("#1a1b26".to_string()), // tokyo night dark bg
    }
}

/// Dracula theme - Popular dark theme with high contrast
fn dracula_theme() -> ThemeConfig {
    ThemeConfig {
        selected_color: Some("#ff79c6".to_string()),  // pink
        directory_color: Some("#8be9fd".to_string()), // cyan
        file_color: Some("#f8f8f2".to_string()),      // white
        border_color: Some("#6272a4".to_string()),    // comment gray
        error_color: Some("#ff5555".to_string()),     // red
        highlight_color: Some("#f1fa8c".to_string()), // yellow
        cursor_color: Some("#bd93f9".to_string()),    // purple for search & bookmarks
        tree_cursor_color: Some("dim".to_string()),   // "dim" = no color, just dimming
        tree_cursor_bg_color: Some("dim".to_string()), // "dim" = no background color
        main_border_color: Some("#6272a4".to_string()), // comment gray border
        panel_border_color: Some("#ff79c6".to_string()), // pink panel borders (search, bookmarks)
        background_color: Some("#282a36".to_string()), // dracula dark bg
    }
}
