use std::path::PathBuf;
use std::time::{Duration, Instant};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind, MouseButton};
use arboard::Clipboard;
use anyhow::Result;

use crate::navigation::Navigation;
use crate::file_viewer::FileViewer;
use crate::search::Search;
use crate::bookmarks::Bookmarks;
use crate::ui::UI;
use crate::config::Config;

/// Event handler for keyboard and mouse input
pub struct EventHandler {
    pub dragging: bool,
    pub last_click_time: Option<(Instant, usize)>,
}

impl EventHandler {
    pub fn new() -> Self {
        Self {
            dragging: false,
            last_click_time: None,
        }
    }

    /// Handle keyboard events
    pub fn handle_key(
        &mut self,
        key: KeyEvent,
        nav: &mut Navigation,
        file_viewer: &mut FileViewer,
        search: &mut Search,
        bookmarks: &mut Bookmarks,
        show_files: &mut bool,
        show_files_before_help: &mut bool,
        show_help: &mut bool,
        fullscreen_viewer: &mut bool,
        ui: &UI,
        config: &Config,
    ) -> Result<Option<PathBuf>> {
        // Search mode - separate handling
        if search.mode {
            return self.handle_search_input(key, search, nav, *show_files);
        }

        // Bookmark selection mode (navigation + filter)
        if bookmarks.is_selecting {
            match key.code {
                KeyCode::Esc => {
                    bookmarks.exit_selection_mode();
                    return Ok(Some(PathBuf::new()));
                }
                KeyCode::Tab => {
                    // Toggle between navigation and filter mode
                    bookmarks.toggle_filter_mode();
                    return Ok(Some(PathBuf::new()));
                }
                KeyCode::Enter => {
                    // Select currently highlighted bookmark (not by name)
                    if let Some(bookmark) = bookmarks.get_selected_bookmark() {
                        let path = bookmark.path.clone();
                        let bookmark_key = bookmark.key.clone();
                        let dir_name = bookmark.name.clone().unwrap_or_else(|| bookmark_key.clone());
                        bookmarks.exit_selection_mode();

                        // Try to navigate and check for errors
                        if let Ok(Some(error_msg)) = nav.go_to_directory(path, *show_files) {
                            // Error occurred - enable file viewer if not already enabled
                            if !*show_files {
                                *show_files = true;
                                nav.reload_tree(*show_files)?;
                            }

                            // Display error details in file viewer
                            let error_content = vec![
                                format!("Error accessing bookmark '{}' ({})", bookmark_key, dir_name),
                                String::new(),
                                error_msg,
                                String::new(),
                                "This directory cannot be accessed. Possible reasons:".to_string(),
                                "- Insufficient permissions".to_string(),
                                "- Directory was removed or renamed".to_string(),
                                "- Filesystem error".to_string(),
                            ];
                            file_viewer.load_content(error_content);
                            *show_help = false;
                        } else {
                            // Success - load file preview if needed
                            if *show_files {
                                if let Some(node) = nav.get_selected_node() {
                                    let _ = ui.load_file_for_viewer(file_viewer, &node.borrow().path, config.behavior.max_file_lines, false, config);
                                }
                            }
                        }
                    } else {
                        // No bookmark selected (empty list) - just exit
                        bookmarks.exit_selection_mode();
                    }
                    return Ok(Some(PathBuf::new()));
                }
                KeyCode::Char('j') | KeyCode::Down if !bookmarks.filter_mode => {
                    // Navigation mode - move down
                    bookmarks.move_down();
                    return Ok(Some(PathBuf::new()));
                }
                KeyCode::Char('k') | KeyCode::Up if !bookmarks.filter_mode => {
                    // Navigation mode - move up
                    bookmarks.move_up();
                    return Ok(Some(PathBuf::new()));
                }
                KeyCode::Char(c) if bookmarks.filter_mode => {
                    // Filter mode - add character and update filter
                    bookmarks.add_char(c);
                    return Ok(Some(PathBuf::new()));
                }
                KeyCode::Backspace if bookmarks.filter_mode => {
                    // Filter mode - remove character and update filter
                    bookmarks.backspace();
                    return Ok(Some(PathBuf::new()));
                }
                _ => {
                    return Ok(Some(PathBuf::new()));
                }
            }
        }

        // Bookmark creation mode (text input for bookmark name)
        if bookmarks.is_creating {
            match key.code {
                KeyCode::Esc => {
                    bookmarks.exit_creation_mode();
                    return Ok(Some(PathBuf::new()));
                }
                KeyCode::Enter => {
                    let bookmark_name = bookmarks.get_input().to_string();
                    if !bookmark_name.is_empty() {
                        if let Some(node) = nav.get_selected_node() {
                            let path = node.borrow().path.clone();
                            let dir_name = path.file_name()
                                .and_then(|n| n.to_str())
                                .map(|s| s.to_string());
                            let _ = bookmarks.add(bookmark_name, path, dir_name);
                        }
                    }
                    bookmarks.exit_creation_mode();
                    return Ok(Some(PathBuf::new()));
                }
                KeyCode::Char(c) => {
                    bookmarks.add_char(c);
                    return Ok(Some(PathBuf::new()));
                }
                KeyCode::Backspace => {
                    bookmarks.backspace();
                    return Ok(Some(PathBuf::new()));
                }
                _ => {
                    return Ok(Some(PathBuf::new()));
                }
            }
        }

        // Handle Ctrl+j/k for scrolling in file viewer or help
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('j') => {
                    if *show_files || *show_help {
                        file_viewer.scroll_down_simple();
                    }
                    return Ok(Some(PathBuf::new()));
                }
                KeyCode::Char('k') => {
                    if *show_files || *show_help {
                        file_viewer.scroll_up();
                    }
                    return Ok(Some(PathBuf::new()));
                }
                _ => {}
            }
        }

        // Handle Esc key - always exits without directory change
        if matches!(key.code, KeyCode::Esc) {
            if *fullscreen_viewer {
                // In fullscreen: Esc exits completely
                return Ok(None);
            } else if search.show_results {
                search.close_results();
                return Ok(Some(PathBuf::new()));
            } else {
                return Ok(None);
            }
        }

        // Handle q key - exits with directory change (except in fullscreen)
        if matches!(key.code, KeyCode::Char('q') | KeyCode::Char('Q')) {
            if *fullscreen_viewer {
                // In fullscreen: q returns to tree view
                *fullscreen_viewer = false;
                return Ok(Some(PathBuf::new()));
            } else {
                // Normal mode: q exits with cd to selected directory
                if let Some(node) = nav.get_selected_node() {
                    let node_borrowed = node.borrow();
                    if node_borrowed.is_dir {
                        return Ok(Some(node_borrowed.path.clone()));
                    }
                }
                return Ok(None);
            }
        }

        match key.code {
            _ if config.keybindings.is_search(key.code) => {
                search.enter_mode();
                return Ok(Some(PathBuf::new()));
            }
            KeyCode::Tab => {
                search.toggle_focus();
                return Ok(Some(PathBuf::new()));
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if search.focus_on_results {
                    search.move_down();
                } else {
                    nav.move_down();
                    if *show_files || *fullscreen_viewer {
                        if let Some(node) = nav.get_selected_node() {
                            let _ = ui.load_file_for_viewer(file_viewer, &node.borrow().path, config.behavior.max_file_lines, *fullscreen_viewer, config);
                            *show_help = false;
                        }
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if search.focus_on_results {
                    search.move_up();
                } else {
                    nav.move_up();
                    if *show_files || *fullscreen_viewer {
                        if let Some(node) = nav.get_selected_node() {
                            let _ = ui.load_file_for_viewer(file_viewer, &node.borrow().path, config.behavior.max_file_lines, *fullscreen_viewer, config);
                            *show_help = false;
                        }
                    }
                }
            }
            KeyCode::Enter => {
                if search.focus_on_results && search.show_results {
                    // In search mode: jump to search result
                    if let Some(path) = search.get_selected_result() {
                        let _ = nav.expand_path_to_node(&path, *show_files);
                        search.focus_on_results = false;
                        if *show_files {
                            let _ = ui.load_file_for_viewer(file_viewer, &path, config.behavior.max_file_lines, false, config);
                            *show_help = false;
                        }
                    }
                    return Ok(Some(PathBuf::new()));
                } else {
                    // Normal mode: Enter on directory -> go inside (change root)
                    if let Some(node) = nav.get_selected_node() {
                        let node_borrowed = node.borrow();
                        if node_borrowed.is_dir {
                            let path = node_borrowed.path.clone();
                            let dir_name = node_borrowed.name.clone();
                            drop(node_borrowed);

                            // Try to navigate and check for errors
                            if let Ok(Some(error_msg)) = nav.go_to_directory(path, *show_files) {
                                // Error occurred - enable file viewer if not already enabled
                                if !*show_files {
                                    *show_files = true;
                                    nav.reload_tree(*show_files)?;
                                }

                                // Display error details in file viewer
                                let error_content = vec![
                                    format!("Error accessing directory: {}", dir_name),
                                    String::new(),
                                    error_msg,
                                    String::new(),
                                    "This directory cannot be accessed. Possible reasons:".to_string(),
                                    "- Insufficient permissions".to_string(),
                                    "- Directory was removed or renamed".to_string(),
                                    "- Filesystem error".to_string(),
                                ];
                                file_viewer.load_content(error_content);
                                *show_help = false;
                            } else {
                                // Success - load file preview if needed
                                if *show_files {
                                    if let Some(node) = nav.get_selected_node() {
                                        let _ = ui.load_file_for_viewer(file_viewer, &node.borrow().path, config.behavior.max_file_lines, false, config);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            KeyCode::Char('l') | KeyCode::Right => {
                if !search.focus_on_results {
                    if let Some(node) = nav.get_selected_node() {
                        let node_borrowed = node.borrow();
                        if node_borrowed.is_dir {
                            let path = node_borrowed.path.clone();
                            let dir_name = node_borrowed.name.clone();
                            drop(node_borrowed);

                            // Toggle node and check for errors
                            if let Ok(Some(error_msg)) = nav.toggle_node(&path, *show_files) {
                                // Error occurred - enable file viewer if not already enabled
                                if !*show_files {
                                    *show_files = true;
                                    nav.reload_tree(*show_files)?;
                                }

                                // Display error details in file viewer
                                let error_content = vec![
                                    format!("Error accessing directory: {}", dir_name),
                                    String::new(),
                                    error_msg,
                                    String::new(),
                                    "This directory cannot be read. Possible reasons:".to_string(),
                                    "- Insufficient permissions".to_string(),
                                    "- Directory was removed or renamed".to_string(),
                                    "- Filesystem error".to_string(),
                                ];
                                file_viewer.load_content(error_content);
                                *show_help = false;
                            }
                        }
                    }
                }
            }
            KeyCode::Char('h') | KeyCode::Left => {
                if let Some(node) = nav.get_selected_node() {
                    let node_borrowed = node.borrow();
                    if node_borrowed.is_dir {
                        let path = node_borrowed.path.clone();
                        drop(node_borrowed);
                        let _ = nav.toggle_node(&path, *show_files)?;
                    }
                }
            }
            KeyCode::Char('u') | KeyCode::Backspace => {
                nav.go_to_parent(*show_files)?;
            }
            _ if config.keybindings.is_toggle_files(key.code) => {
                *show_files = !*show_files;
                *show_help = false;
                nav.reload_tree(*show_files)?;

                // Fix selection if it's out of bounds after reload
                if nav.selected >= nav.flat_list.len() && !nav.flat_list.is_empty() {
                    nav.selected = nav.flat_list.len() - 1;
                }

                if *show_files {
                    if let Some(node) = nav.get_selected_node() {
                        let _ = ui.load_file_for_viewer(file_viewer, &node.borrow().path, config.behavior.max_file_lines, false, config);
                    }
                }
            }
            _ if config.keybindings.is_toggle_help(key.code) => {
                *show_help = !*show_help;

                if *show_help {
                    // Save current show_files state before opening help
                    *show_files_before_help = *show_files;

                    // Load help content into file viewer for scrolling
                    file_viewer.load_content(crate::ui::get_help_content());
                    if !*show_files {
                        *show_files = true;
                        nav.reload_tree(*show_files)?;
                    }
                } else {
                    // Restore previous show_files state
                    if *show_files != *show_files_before_help {
                        *show_files = *show_files_before_help;
                        nav.reload_tree(*show_files)?;
                    }
                    file_viewer.reset_scroll();
                }
            }
            KeyCode::Char('v') => {
                // Toggle fullscreen viewer mode
                if let Some(node) = nav.get_selected_node() {
                    let node_borrowed = node.borrow();
                    if !node_borrowed.is_dir {
                        *fullscreen_viewer = !*fullscreen_viewer;
                        *show_help = false;

                        if *fullscreen_viewer {
                            // Load file for fullscreen viewing with full terminal width
                            let _ = ui.load_file_for_viewer(file_viewer, &node_borrowed.path, config.behavior.max_file_lines, true, config);
                        }
                    }
                }
            }
            _ if config.keybindings.is_copy_path(key.code) => {
                if let Some(node) = nav.get_selected_node() {
                    if let Ok(mut clipboard) = Clipboard::new() {
                        let _ = clipboard.set_text(node.borrow().path.display().to_string());
                    }
                }
            }
            _ if config.keybindings.is_open_editor(key.code) => {
                // Open file in external editor
                if let Some(node) = nav.get_selected_node() {
                    let node_borrowed = node.borrow();
                    if !node_borrowed.is_dir {
                        // Return special marker path to signal editor opening
                        let marker_path = PathBuf::from(format!("EDITOR:{}", node_borrowed.path.display()));
                        return Ok(Some(marker_path));
                    }
                }
            }
            _ if config.keybindings.is_open_file_manager(key.code) => {
                // Open in file manager
                if let Some(node) = nav.get_selected_node() {
                    let node_borrowed = node.borrow();
                    let path_to_open = if node_borrowed.is_dir {
                        // For directories, open the directory itself
                        node_borrowed.path.clone()
                    } else {
                        // For files, open the parent directory
                        node_borrowed.path.parent()
                            .unwrap_or(&node_borrowed.path)
                            .to_path_buf()
                    };
                    // Return special marker path to signal file manager opening
                    let marker_path = PathBuf::from(format!("FILEMGR:{}", path_to_open.display()));
                    return Ok(Some(marker_path));
                }
            }
            _ if config.keybindings.is_create_bookmark(key.code) => {
                // Enter bookmark creation mode
                bookmarks.enter_creation_mode();
            }
            _ if config.keybindings.is_select_bookmark(key.code) => {
                // Enter bookmark selection mode
                bookmarks.enter_selection_mode();
            }
            KeyCode::Char('n') => {
                // Toggle line numbers (only in fullscreen mode)
                if *fullscreen_viewer {
                    file_viewer.toggle_line_numbers();
                }
            }
            KeyCode::PageUp => {
                // Scroll up by page (fullscreen mode only)
                if *fullscreen_viewer {
                    let visible_height = ui.viewer_area_height.saturating_sub(4) as usize;
                    file_viewer.scroll_page_up(visible_height);
                }
            }
            KeyCode::PageDown => {
                // Scroll down by page (fullscreen mode only)
                if *fullscreen_viewer {
                    let visible_height = ui.viewer_area_height.saturating_sub(4) as usize;
                    let max_visible_lines = visible_height.saturating_sub(2);
                    file_viewer.scroll_page_down(visible_height, max_visible_lines);
                }
            }
            KeyCode::Home => {
                // Jump to top of file (fullscreen mode only)
                if *fullscreen_viewer {
                    file_viewer.reset_scroll();
                }
            }
            KeyCode::End => {
                // Jump to end of file (fullscreen mode only)
                if *fullscreen_viewer {
                    let visible_height = ui.viewer_area_height.saturating_sub(4) as usize;
                    file_viewer.scroll_to_end(visible_height);
                }
            }
            _ => {}
        }

        Ok(Some(PathBuf::new()))
    }

    fn handle_search_input(
        &mut self,
        key: KeyEvent,
        search: &mut Search,
        nav: &Navigation,
        show_files: bool,
    ) -> Result<Option<PathBuf>> {
        match key.code {
            KeyCode::Esc => {
                search.exit_mode();
                return Ok(Some(PathBuf::new()));
            }
            KeyCode::Enter => {
                search.perform_search(&nav.root, show_files);
                return Ok(Some(PathBuf::new()));
            }
            KeyCode::Char(c) => {
                search.add_char(c);
                return Ok(Some(PathBuf::new()));
            }
            KeyCode::Backspace => {
                search.backspace();
                return Ok(Some(PathBuf::new()));
            }
            _ => return Ok(Some(PathBuf::new())),
        }
    }

    /// Handle mouse events
    pub fn handle_mouse(
        &mut self,
        mouse: MouseEvent,
        nav: &mut Navigation,
        file_viewer: &mut FileViewer,
        ui: &mut UI,
        show_files: &mut bool,
        show_help: &mut bool,
        fullscreen_viewer: bool,
        config: &Config,
    ) -> Result<()> {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.handle_mouse_click(mouse, nav, file_viewer, ui, show_files, show_help, fullscreen_viewer, config)?;
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                // Ignore dragging in fullscreen mode
                if !fullscreen_viewer && self.dragging && ui.terminal_width > 0 {
                    // Convert mouse position to percentage
                    let new_pos = (mouse.column as u16 * 100) / ui.terminal_width;
                    // Update split position in UI (clamped to 20-80%)
                    ui.adjust_split(new_pos);
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                if !fullscreen_viewer {
                    self.dragging = false;
                }
            }
            MouseEventKind::ScrollUp => {
                self.handle_scroll_up(mouse, nav, file_viewer, ui, show_files, show_help, fullscreen_viewer, config)?;
            }
            MouseEventKind::ScrollDown => {
                self.handle_scroll_down(mouse, nav, file_viewer, ui, show_files, show_help, fullscreen_viewer, config)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_mouse_click(
        &mut self,
        mouse: MouseEvent,
        nav: &mut Navigation,
        file_viewer: &mut FileViewer,
        ui: &mut UI,
        show_files: &mut bool,
        show_help: &mut bool,
        fullscreen_viewer: bool,
        config: &Config,
    ) -> Result<()> {
        // In fullscreen mode, ignore mouse clicks
        if fullscreen_viewer {
            return Ok(());
        }

        // Check click in tree area
        if mouse.column >= ui.tree_area_start && mouse.column < ui.tree_area_end
            && mouse.row >= ui.tree_area_top && mouse.row < ui.tree_area_top + ui.tree_area_height {

            // Calculate clicked row accounting for scroll offset
            let clicked_row_visible = mouse.row.saturating_sub(ui.tree_area_top + 1) as usize;
            let clicked_row = clicked_row_visible + ui.tree_scroll_offset;

            if clicked_row < nav.flat_list.len() {
                let now = Instant::now();
                let is_double_click = if let Some((last_time, last_idx)) = self.last_click_time {
                    clicked_row == last_idx && now.duration_since(last_time) < Duration::from_millis(config.behavior.double_click_timeout_ms)
                } else {
                    false
                };

                if is_double_click {
                    let node = &nav.flat_list[clicked_row];
                    let node_borrowed = node.borrow();
                    if node_borrowed.is_dir {
                        let path = node_borrowed.path.clone();
                        let dir_name = node_borrowed.name.clone();
                        drop(node_borrowed);

                        // Toggle node and check for errors
                        if let Ok(Some(error_msg)) = nav.toggle_node(&path, *show_files) {
                            // Error occurred - enable file viewer if not already enabled
                            if !*show_files {
                                *show_files = true;
                                nav.reload_tree(*show_files)?;
                            }

                            // Display error details in file viewer
                            let error_content = vec![
                                format!("Error accessing directory: {}", dir_name),
                                String::new(),
                                error_msg,
                                String::new(),
                                "This directory cannot be read. Possible reasons:".to_string(),
                                "- Insufficient permissions".to_string(),
                                "- Directory was removed or renamed".to_string(),
                                "- Filesystem error".to_string(),
                            ];
                            file_viewer.load_content(error_content);
                            *show_help = false;
                        }
                    }
                    self.last_click_time = None;
                } else {
                    nav.selected = clicked_row;
                    self.last_click_time = Some((now, clicked_row));

                    if *show_files || fullscreen_viewer {
                        let path = nav.flat_list[clicked_row].borrow().path.clone();
                        let _ = ui.load_file_for_viewer(file_viewer, &path, config.behavior.max_file_lines, fullscreen_viewer, config);
                        *show_help = false;
                    }
                }
            }
        } else if *show_files {
            // Check click on divider
            let divider_col = (ui.terminal_width * ui.split_position) / 100;
            if mouse.column.abs_diff(divider_col) <= 2 {
                self.dragging = true;
            }
        }
        Ok(())
    }

    fn handle_scroll_up(
        &mut self,
        mouse: MouseEvent,
        nav: &mut Navigation,
        file_viewer: &mut FileViewer,
        ui: &mut UI,
        show_files: &mut bool,
        show_help: &mut bool,
        fullscreen_viewer: bool,
        config: &Config,
    ) -> Result<()> {
        // In fullscreen mode, always scroll the file viewer
        if fullscreen_viewer {
            file_viewer.scroll_up();
        } else if (*show_files || *show_help) && mouse.column >= ui.viewer_area_start
            && mouse.row >= ui.viewer_area_top
            && mouse.row < ui.viewer_area_top + ui.viewer_area_height {
            file_viewer.scroll_up();
        } else {
            nav.move_up();
            if (*show_files || fullscreen_viewer) && !*show_help {
                if let Some(node) = nav.get_selected_node() {
                    let _ = ui.load_file_for_viewer(file_viewer, &node.borrow().path, config.behavior.max_file_lines, fullscreen_viewer, config);
                }
            }
        }
        Ok(())
    }

    fn handle_scroll_down(
        &mut self,
        mouse: MouseEvent,
        nav: &mut Navigation,
        file_viewer: &mut FileViewer,
        ui: &mut UI,
        show_files: &mut bool,
        show_help: &mut bool,
        fullscreen_viewer: bool,
        config: &Config,
    ) -> Result<()> {
        // In fullscreen mode, always scroll the file viewer
        if fullscreen_viewer {
            let content_height = ui.viewer_area_height.saturating_sub(2) as usize;
            let lines_to_show = content_height.saturating_sub(2);
            file_viewer.scroll_down(lines_to_show);
        } else if (*show_files || *show_help) && mouse.column >= ui.viewer_area_start
            && mouse.row >= ui.viewer_area_top
            && mouse.row < ui.viewer_area_top + ui.viewer_area_height {
            let content_height = ui.viewer_area_height.saturating_sub(2) as usize;
            let lines_to_show = content_height.saturating_sub(2);
            file_viewer.scroll_down(lines_to_show);
        } else {
            if nav.selected < nav.flat_list.len().saturating_sub(1) {
                nav.move_down();
                if (*show_files || fullscreen_viewer) && !*show_help {
                    if let Some(node) = nav.get_selected_node() {
                        let _ = ui.load_file_for_viewer(file_viewer, &node.borrow().path, config.behavior.max_file_lines, fullscreen_viewer, config);
                    }
                }
            }
        }
        Ok(())
    }
}
