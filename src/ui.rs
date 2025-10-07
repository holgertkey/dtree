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
    pub tree_scroll_offset: usize,
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
            tree_scroll_offset: 0,
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
        fullscreen_viewer: bool,
    ) {
        self.terminal_width = frame.area().width;
        let main_area = frame.area();

        // If in fullscreen viewer mode, render only the file viewer
        if fullscreen_viewer {
            self.render_file_viewer(frame, main_area, file_viewer, false);
            return;
        }

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

        // Calculate scroll offset with margins from top and bottom
        let visible_height = area.height.saturating_sub(2) as usize; // Account for borders
        let total_items = nav.flat_list.len();
        let lines_from_bottom = 5;
        let lines_from_top = 7;

        // Calculate max possible offset (when end of list is visible)
        let max_offset = total_items.saturating_sub(visible_height);

        let final_offset = if max_offset == 0 {
            // List fits entirely in window - no scrolling needed
            0
        } else if nav.selected < lines_from_top {
            // At the beginning: cursor moves freely until line 7
            0
        } else if nav.selected >= total_items.saturating_sub(lines_from_bottom) {
            // At the end: show end of list, cursor moves freely
            max_offset
        } else {
            // In the middle: keep cursor at line 7 from top (or 5 from bottom, whichever comes first)
            let offset_from_top = nav.selected.saturating_sub(lines_from_top);
            let offset_from_bottom = nav.selected.saturating_sub(visible_height.saturating_sub(lines_from_bottom));

            // Use the smaller offset, but not more than max_offset
            offset_from_top.max(offset_from_bottom).min(max_offset)
        };

        *state.offset_mut() = final_offset;
        self.tree_scroll_offset = final_offset;

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

    /// Helper method to load file with correct width for the viewer
    pub fn load_file_for_viewer(&self, file_viewer: &mut FileViewer, path: &std::path::Path) -> anyhow::Result<()> {
        // Calculate available width (accounting for borders and padding)
        let max_width = self.terminal_width
            .saturating_sub(self.split_position * self.terminal_width / 100)
            .saturating_sub(4) as usize; // Account for borders and padding

        file_viewer.load_file_with_width(path, Some(max_width))
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
        } else if area == frame.area() {
            // Fullscreen mode
            format!(" File Viewer (Fullscreen - q/Esc to exit){} ", scroll_info)
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
        "  v              Open file in fullscreen viewer (only for files)".to_string(),
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
        "COMMAND LINE OPTIONS".to_string(),
        "  dtree [PATH]           Navigate directory tree from PATH".to_string(),
        "  dtree -v FILE          View FILE directly in fullscreen mode".to_string(),
        "  dtree --view FILE      View FILE directly in fullscreen mode".to_string(),
        "  dtree --version        Print version information".to_string(),
        "  dtree -h / --help      Print this help message".to_string(),
        "".to_string(),
        "BASH INTEGRATION".to_string(),
        "  The 'dt' wrapper function is recommended for shell integration.".to_string(),
        "  Add to your ~/.bashrc:".to_string(),
        "".to_string(),
        "dt() {".to_string(),
        "  # If flags are passed, just run dtree directly without cd".to_string(),
        "  case \"$1\" in".to_string(),
        "    -h|--help|--version)".to_string(),
        "      command dtree \"$@\"".to_string(),
        "      return".to_string(),
        "      ;;".to_string(),
        "    -v|--view)".to_string(),
        "      command dtree \"$@\"".to_string(),
        "      return".to_string(),
        "      ;;".to_string(),
        "  esac".to_string(),
        "".to_string(),
        "  # If path argument provided, check if it exists".to_string(),
        "  if [ -n \"$1\" ] && [ ! -d \"$1\" ]; then".to_string(),
        "    echo \"Error: Directory '$1' does not exist\" >&2".to_string(),
        "    return 1".to_string(),
        "  fi".to_string(),
        "".to_string(),
        "  local result=$(command dtree \"$@\")".to_string(),
        "  # Only cd if result is a valid directory (ignores errors)".to_string(),
        "  if [ -n \"$result\" ] && [ -d \"$result\" ]; then".to_string(),
        "    cd \"$result\" || return".to_string(),
        "  fi".to_string(),
        "}".to_string(),
        "".to_string(),
        "  Usage:".to_string(),
        "    dt                   Open tree from current directory and cd to selection".to_string(),
        "    dt /path/to/dir      Open tree from specified directory".to_string(),
        "    dt -v file.txt       View file in fullscreen mode".to_string(),
        "    dt --view file.txt   View file in fullscreen mode".to_string(),
        "    dt -h                Show help".to_string(),
        "    dt --version         Show version".to_string(),
        "".to_string(),
        "Press 'i' again to close this help screen".to_string(),
    ]
}
