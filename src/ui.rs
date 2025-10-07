use ratatui::{
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    style::{Modifier, Style, Color},
    layout::{Layout, Constraint, Direction, Rect},
    text::{Line, Span},
    Frame,
};
use crate::tree_node::TreeNodeRef;
use crate::file_viewer::FileViewer;
use crate::navigation::Navigation;
use crate::search::Search;

/// UI rendering module
pub struct UI {
    pub tree_area_start: u16,
    pub tree_area_end: u16,
    pub tree_area_top: u16,
    pub tree_area_height: u16,
    pub viewer_area_start: u16,
    pub viewer_area_top: u16,
    pub viewer_area_height: u16,
    pub terminal_width: u16,
    pub split_position: u16,
}

impl UI {
    pub fn new() -> Self {
        Self {
            tree_area_start: 0,
            tree_area_end: 0,
            tree_area_top: 0,
            tree_area_height: 0,
            viewer_area_start: 0,
            viewer_area_top: 0,
            viewer_area_height: 0,
            terminal_width: 0,
            split_position: 50,
        }
    }

    /// Adjust split position (20-80% range)
    pub fn adjust_split(&mut self, position: u16) {
        self.split_position = position.clamp(20, 80);
    }

    /// Main render function
    pub fn render(
        &mut self,
        frame: &mut Frame,
        nav: &Navigation,
        file_viewer: &FileViewer,
        search: &Search,
        show_files: bool,
        show_help: bool,
    ) {
        self.terminal_width = frame.area().width;
        let main_area = frame.area();

        // Reserve space for search bar if in search mode
        let (content_area, search_bar_area) = if search.mode {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(3),
                    Constraint::Length(3),
                ])
                .split(main_area);
            (chunks[0], Some(chunks[1]))
        } else {
            (main_area, None)
        };

        // If showing search results, split vertically
        let (tree_area, search_results_area) = if search.show_results {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(70),
                    Constraint::Percentage(30),
                ])
                .split(content_area);
            (chunks[0], Some(chunks[1]))
        } else {
            (content_area, None)
        };

        // If file viewer mode enabled, split horizontally
        if show_files {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(self.split_position),
                    Constraint::Percentage(100 - self.split_position),
                ])
                .split(tree_area);

            self.tree_area_start = chunks[0].x;
            self.tree_area_end = chunks[0].x + chunks[0].width;

            self.render_tree(frame, chunks[0], nav);
            self.render_file_viewer(frame, chunks[1], file_viewer, show_help);
        } else {
            self.tree_area_start = tree_area.x;
            self.tree_area_end = tree_area.x + tree_area.width;
            self.render_tree(frame, tree_area, nav);
        }

        // Render search results panel if active
        if let Some(area) = search_results_area {
            self.render_search_results(frame, area, search, &nav.root);
        }

        // Render search bar if in input mode
        if let Some(area) = search_bar_area {
            self.render_search_bar(frame, area, search);
        }
    }

    fn render_tree(&mut self, frame: &mut Frame, area: Rect, nav: &Navigation) {
        self.tree_area_top = area.y;
        self.tree_area_height = area.height;

        let items: Vec<ListItem> = nav.flat_list.iter().map(|node| {
            let node_borrowed = node.borrow();
            let indent = "  ".repeat(node_borrowed.depth);

            // Icon with error indicator
            let icon = if node_borrowed.has_error {
                if node_borrowed.is_dir {
                    if node_borrowed.is_expanded { "⚠ " } else { "⚠ " }
                } else {
                    "⚠ "
                }
            } else if node_borrowed.is_dir {
                if node_borrowed.is_expanded { "▼ " } else { "▶ " }
            } else {
                "  "
            };

            // Add error message to name if present
            let name_with_error = if let Some(ref error_msg) = node_borrowed.error_message {
                format!("{} [{}]", node_borrowed.name, error_msg)
            } else {
                node_borrowed.name.clone()
            };

            let text = format!("{}{}{}", indent, icon, name_with_error);

            // Color coding: errors in red, directories in white, files in gray
            let style = if node_borrowed.has_error {
                Style::default().fg(Color::Red)
            } else if node_borrowed.is_dir {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            ListItem::new(text).style(style)
        }).collect();

        let mut state = ListState::default();
        state.select(Some(nav.selected));

        // Calculate scroll offset: start scrolling when cursor is below line 17
        let scroll_threshold = 17;
        let visible_height = area.height.saturating_sub(2) as usize; // Account for borders
        let total_items = nav.flat_list.len();

        if nav.selected > scroll_threshold {
            let offset = nav.selected.saturating_sub(scroll_threshold);

            // Stop scrolling when the end of the list is visible
            let max_offset = total_items.saturating_sub(visible_height);
            let final_offset = offset.min(max_offset);

            *state.offset_mut() = final_offset;
        }

        let title = " Directory Tree (↑↓/jk: navigate | /: search | c: copy | Enter: select | q: quit | i: help) ";

        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title))
            .highlight_style(Style::default()
                .add_modifier(Modifier::DIM))
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, area, &mut state);
    }

    fn render_search_bar(&self, frame: &mut Frame, area: Rect, search: &Search) {
        let search_text = format!("Search: {}", search.query);

        let paragraph = Paragraph::new(search_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(" Enter to search, Esc to cancel "))
            .style(Style::default().fg(Color::Cyan));

        frame.render_widget(paragraph, area);
    }

    fn render_search_results(&self, frame: &mut Frame, area: Rect, search: &Search, root: &TreeNodeRef) {
        let root_path = root.borrow().path.clone();
        let root_parent = root_path.parent().unwrap_or(&root_path);

        let items: Vec<ListItem> = search.results.iter().map(|path| {
            let display_path = path.strip_prefix(root_parent)
                .unwrap_or(path)
                .display()
                .to_string();

            let style = Style::default().fg(Color::White);
            ListItem::new(display_path).style(style)
        }).collect();

        let mut state = ListState::default();
        state.select(Some(search.selected));

        let title = format!(" Search Results: {} found (Enter to select, q to close) ",
            search.results.len());

        let border_style = if search.focus_on_results {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };

        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style))
            .highlight_style(Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, area, &mut state);
    }

    fn render_file_viewer(&mut self, frame: &mut Frame, area: Rect, file_viewer: &FileViewer, show_help: bool) {
        self.viewer_area_start = area.x;
        self.viewer_area_top = area.y;
        self.viewer_area_height = area.height;

        let content_height = area.height.saturating_sub(2) as usize;

        let content_to_display = if show_help {
            get_help_content()
        } else {
            file_viewer.content.clone()
        };

        // Calculate visible lines (leaving space for separator and file info)
        let lines_to_show = content_height.saturating_sub(2);

        let mut visible_lines: Vec<Line> = content_to_display
            .iter()
            .skip(file_viewer.scroll)
            .take(lines_to_show)
            .map(|line| Line::from(line.as_str()))
            .collect();

        // Add separator and file info at the end (only if not help)
        if !show_help && !file_viewer.current_path.as_os_str().is_empty() {
            let file_info = file_viewer.format_file_info();
            let separator = "─".repeat(area.width.saturating_sub(2) as usize);

            visible_lines.push(Line::from(
                Span::styled(separator, Style::default().fg(Color::DarkGray))
            ));
            visible_lines.push(Line::from(
                Span::styled(file_info, Style::default().fg(Color::DarkGray))
            ));
        }

        let scroll_info = if content_to_display.len() > lines_to_show {
            format!(" [↕ {}/{}]", file_viewer.scroll + 1, content_to_display.len())
        } else {
            String::new()
        };

        let title = if show_help {
            format!(" Help{} ", scroll_info)
        } else {
            format!(" File Viewer{} ", scroll_info)
        };

        let paragraph = Paragraph::new(visible_lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title));

        frame.render_widget(paragraph, area);
    }
}

pub fn get_help_content() -> Vec<String> {
    vec![
        "DTREE - Interactive Directory Tree Navigator".to_string(),
        "".to_string(),
        "DESCRIPTION".to_string(),
        "  dtree is a lightweight TUI application for interactive directory tree".to_string(),
        "  navigation. It provides a visual tree view with file preview capabilities.".to_string(),
        "".to_string(),
        "KEYBOARD NAVIGATION".to_string(),
        "  ↑ / k          Navigate up in the tree".to_string(),
        "  ↓ / j          Navigate down in the tree".to_string(),
        "  → / l          Expand directory (show subdirectories)".to_string(),
        "  ← / h          Collapse directory (hide subdirectories)".to_string(),
        "  u              Go to parent directory (change root)".to_string(),
        "  Backspace      Go to parent directory (change root)".to_string(),
        "  Enter          Select directory and exit (cd to selected)".to_string(),
        "  q / Esc        Quit without selection".to_string(),
        "  s              Toggle file viewer mode (show/hide files)".to_string(),
        "  c              Copy current path to clipboard (files and directories)".to_string(),
        "  i              Show/hide this help screen".to_string(),
        "".to_string(),
        "SEARCH".to_string(),
        "  /              Enter search mode".to_string(),
        "  Type query     Type your search query (case-insensitive)".to_string(),
        "  Enter          Execute search and show results panel".to_string(),
        "  Esc            Cancel search (in search mode) or close results panel".to_string(),
        "  q              Close search results panel (when panel is open)".to_string(),
        "".to_string(),
        "  In Search Results Panel:".to_string(),
        "  Tab            Switch focus between tree and search results".to_string(),
        "  ↑↓ / jk        Navigate through search results".to_string(),
        "  Enter          Select result and jump to it in the tree".to_string(),
        "".to_string(),
        "  Search features:".to_string(),
        "  • Search scope: from current root directory and below".to_string(),
        "  • Normal mode: searches ONLY directories (fast)".to_string(),
        "  • File viewer mode (s): searches both files and directories".to_string(),
        "  • Searches through the ENTIRE tree (including collapsed nodes)".to_string(),
        "  • Shows all results in a separate panel at the bottom".to_string(),
        "  • Select a result to automatically expand and jump to it in the tree".to_string(),
        "  • Case-insensitive substring matching".to_string(),
        "  • Cyan border indicates which panel has focus".to_string(),
        "".to_string(),
        "FILE VIEWER MODE (press 's' to toggle)".to_string(),
        "  When enabled:".to_string(),
        "    • Shows files in addition to directories".to_string(),
        "    • Displays file preview panel on the right".to_string(),
        "    • Shows file content (first 1000 lines)".to_string(),
        "    • Displays file information (size, lines, permissions)".to_string(),
        "".to_string(),
        "  File Preview Navigation:".to_string(),
        "    Ctrl+j       Scroll down in file preview".to_string(),
        "    Ctrl+k       Scroll up in file preview".to_string(),
        "    Scroll wheel Scroll file preview (when mouse over preview area)".to_string(),
        "".to_string(),
        "MOUSE SUPPORT".to_string(),
        "  Click          Select item in tree".to_string(),
        "  Double-click   Expand/collapse directory".to_string(),
        "  Scroll wheel   Navigate tree (when mouse over tree area)".to_string(),
        "                 Scroll file preview (when mouse over preview area)".to_string(),
        "  Drag           Resize split view (drag the vertical divider)".to_string(),
        "".to_string(),
        "Press 'i' again to close this help screen".to_string(),
    ]
}
