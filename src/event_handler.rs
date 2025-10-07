use std::path::PathBuf;
use std::time::{Duration, Instant};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind, MouseButton};
use arboard::Clipboard;
use anyhow::Result;

use crate::navigation::Navigation;
use crate::file_viewer::FileViewer;
use crate::search::Search;
use crate::ui::UI;

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
        show_files: &mut bool,
        show_help: &mut bool,
    ) -> Result<Option<PathBuf>> {
        // Search mode - separate handling
        if search.mode {
            return self.handle_search_input(key, search, nav);
        }

        // Handle Ctrl+j/k for scrolling in file viewer
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('j') => {
                    if *show_files {
                        let content_len = if *show_help {
                            crate::ui::get_help_content().len()
                        } else {
                            file_viewer.content.len()
                        };
                        file_viewer.scroll_down(content_len.saturating_sub(1));
                    }
                    return Ok(Some(PathBuf::new()));
                }
                KeyCode::Char('k') => {
                    if *show_files {
                        file_viewer.scroll_up();
                    }
                    return Ok(Some(PathBuf::new()));
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Char('q') => {
                if search.show_results {
                    search.close_results();
                    return Ok(Some(PathBuf::new()));
                } else {
                    return Ok(None);
                }
            }
            KeyCode::Esc => {
                if search.show_results {
                    search.close_results();
                    return Ok(Some(PathBuf::new()));
                } else {
                    return Ok(None);
                }
            }
            KeyCode::Char('/') => {
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
                    if *show_files {
                        if let Some(node) = nav.get_selected_node() {
                            let _ = file_viewer.load_file(&node.borrow().path);
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
                    if *show_files {
                        if let Some(node) = nav.get_selected_node() {
                            let _ = file_viewer.load_file(&node.borrow().path);
                            *show_help = false;
                        }
                    }
                }
            }
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                if key.code == KeyCode::Enter {
                    if search.focus_on_results && search.show_results {
                        if let Some(path) = search.get_selected_result() {
                            let _ = nav.expand_path_to_node(&path, *show_files);
                            search.focus_on_results = false;
                            if *show_files {
                                let _ = file_viewer.load_file(&path);
                                *show_help = false;
                            }
                        }
                        return Ok(Some(PathBuf::new()));
                    } else {
                        if let Some(node) = nav.get_selected_node() {
                            let node_borrowed = node.borrow();
                            if node_borrowed.is_dir {
                                return Ok(Some(node_borrowed.path.clone()));
                            }
                        }
                    }
                } else {
                    if !search.focus_on_results {
                        if let Some(node) = nav.get_selected_node() {
                            let node_borrowed = node.borrow();
                            if node_borrowed.is_dir {
                                let path = node_borrowed.path.clone();
                                drop(node_borrowed);
                                nav.toggle_node(&path, *show_files)?;
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
                        nav.toggle_node(&path, *show_files)?;
                    }
                }
            }
            KeyCode::Char('u') | KeyCode::Backspace => {
                nav.go_to_parent(*show_files)?;
            }
            KeyCode::Char('s') => {
                *show_files = !*show_files;
                *show_help = false;
                nav.reload_tree(*show_files)?;

                if *show_files {
                    if let Some(node) = nav.get_selected_node() {
                        let _ = file_viewer.load_file(&node.borrow().path);
                    }
                }
            }
            KeyCode::Char('i') => {
                *show_help = !*show_help;
                file_viewer.reset_scroll();

                if *show_help && !*show_files {
                    *show_files = true;
                    nav.reload_tree(*show_files)?;
                }
            }
            KeyCode::Char('c') => {
                if let Some(node) = nav.get_selected_node() {
                    if let Ok(mut clipboard) = Clipboard::new() {
                        let _ = clipboard.set_text(node.borrow().path.display().to_string());
                    }
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
    ) -> Result<Option<PathBuf>> {
        match key.code {
            KeyCode::Esc => {
                search.exit_mode();
                return Ok(Some(PathBuf::new()));
            }
            KeyCode::Enter => {
                search.perform_search(&nav.root, false);
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
        show_files: bool,
        show_help: &mut bool,
    ) -> Result<()> {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.handle_mouse_click(mouse, nav, file_viewer, ui, show_files, show_help)?;
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                if self.dragging && ui.terminal_width > 0 {
                    // Convert mouse position to percentage
                    let new_pos = (mouse.column as u16 * 100) / ui.terminal_width;
                    // Update split position in UI (clamped to 20-80%)
                    ui.adjust_split(new_pos);
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                self.dragging = false;
            }
            MouseEventKind::ScrollUp => {
                self.handle_scroll_up(mouse, nav, file_viewer, ui, show_files, show_help)?;
            }
            MouseEventKind::ScrollDown => {
                self.handle_scroll_down(mouse, nav, file_viewer, ui, show_files, show_help)?;
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
        show_files: bool,
        show_help: &mut bool,
    ) -> Result<()> {
        // Check click in tree area
        if mouse.column >= ui.tree_area_start && mouse.column < ui.tree_area_end
            && mouse.row >= ui.tree_area_top && mouse.row < ui.tree_area_top + ui.tree_area_height {

            let clicked_row = mouse.row.saturating_sub(ui.tree_area_top + 1) as usize;

            if clicked_row < nav.flat_list.len() {
                let now = Instant::now();
                let is_double_click = if let Some((last_time, last_idx)) = self.last_click_time {
                    clicked_row == last_idx && now.duration_since(last_time) < Duration::from_millis(500)
                } else {
                    false
                };

                if is_double_click {
                    let node = &nav.flat_list[clicked_row];
                    let node_borrowed = node.borrow();
                    if node_borrowed.is_dir {
                        let path = node_borrowed.path.clone();
                        drop(node_borrowed);
                        nav.toggle_node(&path, show_files)?;
                    }
                    self.last_click_time = None;
                } else {
                    nav.selected = clicked_row;
                    self.last_click_time = Some((now, clicked_row));

                    if show_files {
                        let path = nav.flat_list[clicked_row].borrow().path.clone();
                        let _ = file_viewer.load_file(&path);
                        *show_help = false;
                    }
                }
            }
        } else if show_files {
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
        show_files: bool,
        show_help: &mut bool,
    ) -> Result<()> {
        if show_files && mouse.column >= ui.viewer_area_start
            && mouse.row >= ui.viewer_area_top
            && mouse.row < ui.viewer_area_top + ui.viewer_area_height {
            file_viewer.scroll_up();
        } else {
            nav.move_up();
            if show_files {
                if let Some(node) = nav.get_selected_node() {
                    let _ = file_viewer.load_file(&node.borrow().path);
                    *show_help = false;
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
        show_files: bool,
        show_help: &mut bool,
    ) -> Result<()> {
        if show_files && mouse.column >= ui.viewer_area_start
            && mouse.row >= ui.viewer_area_top
            && mouse.row < ui.viewer_area_top + ui.viewer_area_height {
            let content_height = ui.viewer_area_height.saturating_sub(2) as usize;
            let lines_to_show = content_height.saturating_sub(2);
            file_viewer.scroll_down(lines_to_show);
        } else {
            if nav.selected < nav.flat_list.len().saturating_sub(1) {
                nav.move_down();
                if show_files {
                    if let Some(node) = nav.get_selected_node() {
                        let _ = file_viewer.load_file(&node.borrow().path);
                        *show_help = false;
                    }
                }
            }
        }
        Ok(())
    }
}
