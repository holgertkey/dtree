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
        selected_color: "cyan".to_string(),
        directory_color: "blue".to_string(),
        file_color: "white".to_string(),
        border_color: "gray".to_string(),
        error_color: "red".to_string(),
        highlight_color: "yellow".to_string(),
        cursor_color: "yellow".to_string(),     // for search & bookmarks
        tree_cursor_color: "dim".to_string(),   // "dim" = no color, just dimming
        main_border_color: "gray".to_string(),  // main window border
        panel_border_color: "cyan".to_string(), // panel borders (search, bookmarks)
        background_color: "reset".to_string(),  // terminal default
    }
}

/// Gruvbox theme - Warm, high contrast theme inspired by Gruvbox
fn gruvbox_theme() -> ThemeConfig {
    ThemeConfig {
        selected_color: "#fe8019".to_string(),  // bright orange
        directory_color: "#83a598".to_string(), // bright blue
        file_color: "#ebdbb2".to_string(),      // light foreground
        border_color: "#928374".to_string(),    // gray
        error_color: "#fb4934".to_string(),     // bright red
        highlight_color: "#fabd2f".to_string(), // bright yellow
        cursor_color: "#fabd2f".to_string(),    // yellow for search & bookmarks
        tree_cursor_color: "dim".to_string(),   // "dim" = no color, just dimming
        main_border_color: "#928374".to_string(), // gray border
        panel_border_color: "#fe8019".to_string(), // orange panel borders (search, bookmarks)
        background_color: "#282828".to_string(), // gruvbox dark bg
    }
}

/// Nord theme - Cold, muted colors inspired by Nord theme
fn nord_theme() -> ThemeConfig {
    ThemeConfig {
        selected_color: "#88c0d0".to_string(), // frost cyan
        directory_color: "#81a1c1".to_string(), // frost blue
        file_color: "#eceff4".to_string(),     // snow white
        border_color: "#4c566a".to_string(),   // polar night gray
        error_color: "#bf616a".to_string(),    // aurora red
        highlight_color: "#ebcb8b".to_string(), // aurora yellow
        cursor_color: "#ebcb8b".to_string(),   // yellow for search & bookmarks
        tree_cursor_color: "dim".to_string(),  // "dim" = no color, just dimming
        main_border_color: "#4c566a".to_string(), // polar night gray border
        panel_border_color: "#88c0d0".to_string(), // cyan panel borders (search, bookmarks)
        background_color: "#2e3440".to_string(), // nord dark bg
    }
}
