use ratatui::{
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    style::{Modifier, Style},
    layout::{Layout, Constraint, Direction, Rect},
    text::{Line, Span},
    Frame,
};
use crate::tree_node::TreeNodeRef;
use crate::file_viewer::FileViewer;
use crate::navigation::Navigation;
use crate::search::Search;
use crate::bookmarks::Bookmarks;
use crate::config::Config;

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
        bookmarks: &Bookmarks,
        config: &Config,
        show_files: bool,
        show_help: bool,
        fullscreen_viewer: bool,
    ) {
        self.terminal_width = frame.area().width;
        let main_area = frame.area();

        // If in fullscreen viewer mode, render only the file viewer
        if fullscreen_viewer {
            self.render_file_viewer(frame, main_area, file_viewer, false, config);
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

            self.render_tree(frame, chunks[0], nav, config);
            self.render_file_viewer(frame, chunks[1], file_viewer, show_help, config);
        } else {
            self.tree_area_start = tree_area.x;
            self.tree_area_end = tree_area.x + tree_area.width;
            self.render_tree(frame, tree_area, nav, config);
        }

        // Render search results panel if active
        if let Some(area) = search_results_area {
            self.render_search_results(frame, area, search, &nav.root, config);
        }

        // Render search bar if in input mode
        if let Some(area) = search_bar_area {
            self.render_search_bar(frame, area, search, config);
        }

        // Render bookmarks popup if in selection or creation mode
        if bookmarks.is_selecting || bookmarks.is_creating {
            self.render_bookmarks_popup(frame, bookmarks, config);
        }
    }

    fn render_tree(&mut self, frame: &mut Frame, area: Rect, nav: &Navigation, config: &Config) {
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

            // Just show the name - detailed error info will be in file viewer
            let text = format!("{}{}{}", indent, icon, node_borrowed.name);

            // Color coding: errors in configured color, directories and files use theme colors
            let style = if node_borrowed.has_error {
                let error_color = Config::parse_color(&config.appearance.colors.error_color);
                Style::default().fg(error_color)
            } else if node_borrowed.is_dir {
                let dir_color = Config::parse_color(&config.appearance.colors.directory_color);
                Style::default().fg(dir_color)
            } else {
                let file_color = Config::parse_color(&config.appearance.colors.file_color);
                Style::default().fg(file_color)
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

        let title = " Directory Tree (↑↓/jk: navigate | Enter: go in | q: cd & exit | Esc: exit | /: search | i: help) ";

        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title))
            .highlight_style(Style::default()
                .add_modifier(Modifier::DIM))
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, area, &mut state);
    }

    fn render_search_bar(&self, frame: &mut Frame, area: Rect, search: &Search, config: &Config) {
        let search_text = format!("Search: {}", search.query);

        let selected_color = Config::parse_color(&config.appearance.colors.selected_color);

        let paragraph = Paragraph::new(search_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(" Enter to search, Esc to cancel "))
            .style(Style::default().fg(selected_color));

        frame.render_widget(paragraph, area);
    }

    fn render_search_results(&self, frame: &mut Frame, area: Rect, search: &Search, root: &TreeNodeRef, config: &Config) {
        let root_path = root.borrow().path.clone();
        let root_parent = root_path.parent().unwrap_or(&root_path);

        let file_color = Config::parse_color(&config.appearance.colors.file_color);
        let selected_color = Config::parse_color(&config.appearance.colors.selected_color);
        let highlight_color = Config::parse_color(&config.appearance.colors.highlight_color);

        let items: Vec<ListItem> = search.results.iter().map(|path| {
            let display_path = path.strip_prefix(root_parent)
                .unwrap_or(path)
                .display()
                .to_string();

            let style = Style::default().fg(file_color);
            ListItem::new(display_path).style(style)
        }).collect();

        let mut state = ListState::default();
        state.select(Some(search.selected));

        let title = format!(" Search Results: {} found (Enter to select, q to close) ",
            search.results.len());

        let border_style = if search.focus_on_results {
            Style::default().fg(selected_color)
        } else {
            Style::default()
        };

        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style))
            .highlight_style(Style::default()
                .fg(highlight_color)
                .add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, area, &mut state);
    }

    /// Helper method to load file with correct width for the viewer
    pub fn load_file_for_viewer(&self, file_viewer: &mut FileViewer, path: &std::path::Path, max_lines: usize, fullscreen: bool, config: &Config) -> anyhow::Result<()> {
        let enable_highlighting = config.appearance.enable_syntax_highlighting;
        let theme = &config.appearance.syntax_theme;

        if fullscreen {
            // For fullscreen, use None to get maximum width (no truncation)
            file_viewer.load_file_with_width(path, None, max_lines, enable_highlighting, theme)
        } else {
            // For split view, calculate available width based on split position
            let max_width = self.terminal_width
                .saturating_sub(self.split_position * self.terminal_width / 100)
                .saturating_sub(4) as usize;
            file_viewer.load_file_with_width(path, Some(max_width), max_lines, enable_highlighting, theme)
        }
    }

    fn render_file_viewer(&mut self, frame: &mut Frame, area: Rect, file_viewer: &FileViewer, show_help: bool, config: &Config) {
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

        // Check if we're in fullscreen mode (area == frame.area())
        let is_fullscreen = area == frame.area();
        let show_numbers = is_fullscreen && file_viewer.show_line_numbers && !show_help;

        // Use highlighted content if available, otherwise fall back to plain text
        let use_highlighting = !file_viewer.highlighted_content.is_empty() && !show_help;

        let mut visible_lines: Vec<Line> = if use_highlighting {
            // Use pre-highlighted content
            file_viewer.highlighted_content
                .iter()
                .enumerate()
                .skip(file_viewer.scroll)
                .take(lines_to_show)
                .map(|(idx, line)| {
                    if show_numbers {
                        // Add line numbers to highlighted lines
                        let line_num = file_viewer.scroll + idx + 1;
                        let border_color = Config::parse_color(&config.appearance.colors.border_color);
                        let mut spans = vec![
                            Span::styled(format!("{:4} ", line_num), Style::default().fg(border_color)),
                        ];
                        spans.extend(line.spans.iter().cloned());
                        Line::from(spans)
                    } else {
                        line.clone()
                    }
                })
                .collect()
        } else {
            // Fallback to plain text
            content_to_display
                .iter()
                .enumerate()
                .skip(file_viewer.scroll)
                .take(lines_to_show)
                .map(|(idx, line)| {
                    if show_numbers {
                        // Add line numbers (1-indexed, starting from scroll position)
                        let line_num = file_viewer.scroll + idx + 1;
                        let border_color = Config::parse_color(&config.appearance.colors.border_color);
                        Line::from(vec![
                            Span::styled(format!("{:4} ", line_num), Style::default().fg(border_color)),
                            Span::raw(line.as_str()),
                        ])
                    } else {
                        Line::from(line.as_str())
                    }
                })
                .collect()
        };

        // Add separator and file info at the end (only if not help)
        if !show_help && !file_viewer.current_path.as_os_str().is_empty() {
            let file_info = file_viewer.format_file_info();
            let separator = "─".repeat(area.width.saturating_sub(2) as usize);

            let border_color = Config::parse_color(&config.appearance.colors.border_color);

            visible_lines.push(Line::from(
                Span::styled(separator, Style::default().fg(border_color))
            ));
            visible_lines.push(Line::from(
                Span::styled(file_info, Style::default().fg(border_color))
            ));
        }

        let scroll_info = if content_to_display.len() > lines_to_show {
            format!(" [↕ {}/{}]", file_viewer.scroll + 1, content_to_display.len())
        } else {
            String::new()
        };

        let title = if show_help {
            format!(" Help{} ", scroll_info)
        } else if is_fullscreen {
            // Fullscreen mode
            let line_num_hint = if file_viewer.show_line_numbers { "n: hide lines" } else { "n: show lines" };
            format!(" File Viewer (Fullscreen - {} | q: back | Esc: exit){} ", line_num_hint, scroll_info)
        } else {
            format!(" File Viewer{} ", scroll_info)
        };

        // In fullscreen mode, only show top and bottom borders (no sides)
        let borders = if is_fullscreen {
            Borders::TOP | Borders::BOTTOM
        } else {
            Borders::ALL
        };

        let paragraph = Paragraph::new(visible_lines)
            .block(Block::default()
                .borders(borders)
                .title(title));

        frame.render_widget(paragraph, area);
    }

    fn render_bookmarks_popup(&self, frame: &mut Frame, bookmarks: &Bookmarks, config: &Config) {
        let border_color = Config::parse_color(&config.appearance.colors.border_color);
        let selected_color = Config::parse_color(&config.appearance.colors.selected_color);
        let highlight_color = Config::parse_color(&config.appearance.colors.highlight_color);

        // Center popup in the screen
        let area = frame.area();
        let popup_width = 60.min(area.width.saturating_sub(4));
        let popup_height = 20.min(area.height.saturating_sub(4));

        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;

        let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

        // Build content based on mode
        let (title, items) = if bookmarks.is_creating {
            let title = " Create Bookmark - Enter name and press Enter ";
            let mut lines = vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("Bookmark name: ", Style::default().fg(highlight_color)),
                    Span::styled(bookmarks.get_input(), Style::default().fg(selected_color).add_modifier(Modifier::BOLD)),
                    Span::styled("█", Style::default().fg(selected_color)),  // cursor
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Press Enter to save, Esc to cancel", Style::default().fg(border_color))
                ]),
                Line::from(""),
            ];

            // Show existing bookmarks
            if !bookmarks.list().is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("Existing bookmarks:", Style::default().fg(selected_color).add_modifier(Modifier::BOLD))
                ]));
                lines.push(Line::from(""));

                for bookmark in bookmarks.list() {
                    let name = bookmark.name.as_deref().unwrap_or("(unnamed)");
                    let path_str = bookmark.path.display().to_string();
                    lines.push(Line::from(vec![
                        Span::styled(format!("  {} ", bookmark.key), Style::default().fg(selected_color).add_modifier(Modifier::BOLD)),
                        Span::styled("→ ", Style::default().fg(border_color)),
                        Span::styled(name, Style::default().fg(highlight_color)),
                        Span::styled(format!(" ({})", path_str), Style::default().fg(border_color)),
                    ]));
                }
            } else {
                lines.push(Line::from(vec![
                    Span::styled("No bookmarks yet", Style::default().fg(border_color))
                ]));
            }

            (title, lines)
        } else {
            // Selection mode - text input for bookmark name
            let title = " Select Bookmark - Enter name and press Enter ";
            let mut lines = vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("Bookmark name: ", Style::default().fg(highlight_color)),
                    Span::styled(bookmarks.get_input(), Style::default().fg(selected_color).add_modifier(Modifier::BOLD)),
                    Span::styled("█", Style::default().fg(selected_color)),  // cursor
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Press Enter to jump, Esc to cancel", Style::default().fg(border_color))
                ]),
                Line::from(""),
            ];

            if bookmarks.list().is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("No bookmarks saved yet", Style::default().fg(border_color))
                ]));
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("Press ", Style::default()),
                    Span::styled("m", Style::default().fg(selected_color).add_modifier(Modifier::BOLD)),
                    Span::styled(" to create a bookmark", Style::default()),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled("Available bookmarks:", Style::default().fg(selected_color).add_modifier(Modifier::BOLD))
                ]));
                lines.push(Line::from(""));

                for bookmark in bookmarks.list() {
                    let name = bookmark.name.as_deref().unwrap_or("(unnamed)");
                    let path_str = bookmark.path.display().to_string();
                    lines.push(Line::from(vec![
                        Span::styled(format!("  {} ", bookmark.key), Style::default().fg(selected_color).add_modifier(Modifier::BOLD)),
                        Span::styled("→ ", Style::default().fg(border_color)),
                        Span::styled(name, Style::default().fg(highlight_color)),
                        Span::styled(format!(" ({})", path_str), Style::default().fg(border_color)),
                    ]));
                }
            }

            (title, lines)
        };

        let paragraph = Paragraph::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title)
                .style(Style::default().fg(border_color)));

        frame.render_widget(paragraph, popup_area);
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
        "  Enter          Go into directory (change root to selected directory)".to_string(),
        "  q              Exit and cd to selected directory".to_string(),
        "  Esc            Quit without directory change".to_string(),
        "  s              Toggle file viewer mode (show/hide files)".to_string(),
        "  v              Open file in fullscreen viewer (only for files)".to_string(),
        "  c              Copy current path to clipboard (files and directories)".to_string(),
        "  e              Open file in external editor (configurable in config.toml)".to_string(),
        "  o              Open in file manager (files open parent dir, dirs open themselves)".to_string(),
        "  i              Show/hide this help screen".to_string(),
        "".to_string(),
        "SEARCH".to_string(),
        "  /              Enter search mode".to_string(),
        "  Type query     Type your search query (case-insensitive)".to_string(),
        "  Enter          Execute search and show results panel".to_string(),
        "  Esc            Cancel search (in search mode) or close results panel".to_string(),
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
        "BOOKMARKS".to_string(),
        "  Interactive mode (inside dtree):".to_string(),
        "    m            Enter bookmark creation mode".to_string(),
        "    '            Open bookmark selection menu (tick/apostrophe)".to_string(),
        "".to_string(),
        "  Bookmark Creation (after pressing 'm'):".to_string(),
        "    • Enter bookmark name (multi-character names supported)".to_string(),
        "    • Press Enter to save, Esc to cancel".to_string(),
        "    • Examples: work, project-123, my_home".to_string(),
        "".to_string(),
        "  Bookmark Navigation (after pressing '''):".to_string(),
        "    • List shows all saved bookmarks with their names".to_string(),
        "    • Enter bookmark name (multi-character names supported)".to_string(),
        "    • Press Enter to jump, Esc to cancel".to_string(),
        "    • Examples: work, project-123, my_home".to_string(),
        "".to_string(),
        "  Command-line mode (outside dtree):".to_string(),
        "    dt myproject        Jump to bookmark 'myproject' (if exists)".to_string(),
        "    dt -bm              List all bookmarks".to_string(),
        "    dt -bm list         List all bookmarks".to_string(),
        "    dt -bm add work     Save current directory as 'work'".to_string(),
        "    dt -bm add work /path   Save specific path as 'work'".to_string(),
        "    dt -bm remove work  Remove bookmark 'work'".to_string(),
        "".to_string(),
        "  Storage: ~/.config/dtree/bookmarks.json".to_string(),
        "  Priority: Bookmark names are checked before directory names".to_string(),
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
        "FULLSCREEN VIEWER (press 'v' on a file)".to_string(),
        "  When viewing a file in fullscreen mode:".to_string(),
        "    q            Return to tree view (stay in program)".to_string(),
        "    Esc          Exit program completely (return to terminal)".to_string(),
        "    n            Toggle line numbers (show/hide)".to_string(),
        "".to_string(),
        "  Navigation (fullscreen mode):".to_string(),
        "    Ctrl+j/k     Scroll by line (fine control)".to_string(),
        "    Page Up/Down Scroll by page (fast navigation)".to_string(),
        "    Home         Jump to top of file".to_string(),
        "    End          Jump to bottom of file".to_string(),
        "    Scroll wheel Scroll by line with mouse".to_string(),
        "".to_string(),
        "  Text Selection (fullscreen mode):".to_string(),
        "    Shift+Mouse  Select text (bypasses mouse capture)".to_string(),
        "                 Copy with Ctrl+Shift+C or right-click menu".to_string(),
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
        "  # If flags or bookmark commands are passed, run dtree directly".to_string(),
        "  case \"$1\" in".to_string(),
        "    -h|--help|--version)".to_string(),
        "      command dtree \"$@\"".to_string(),
        "      return".to_string(),
        "      ;;".to_string(),
        "    -v|--view)".to_string(),
        "      command dtree \"$@\"".to_string(),
        "      return".to_string(),
        "      ;;".to_string(),
        "    -bm)".to_string(),
        "      # Bookmark management - run directly".to_string(),
        "      command dtree \"$@\"".to_string(),
        "      return".to_string(),
        "      ;;".to_string(),
        "  esac".to_string(),
        "".to_string(),
        "  # For navigation: dtree resolves paths/bookmarks".to_string(),
        "  local result=$(command dtree \"$@\")".to_string(),
        "  local exit_code=$?".to_string(),
        "".to_string(),
        "  if [ $exit_code -ne 0 ]; then".to_string(),
        "    return $exit_code".to_string(),
        "  fi".to_string(),
        "".to_string(),
        "  # Only cd if result is a valid directory".to_string(),
        "  if [ -n \"$result\" ] && [ -d \"$result\" ]; then".to_string(),
        "    cd \"$result\" || return".to_string(),
        "  fi".to_string(),
        "}".to_string(),
        "".to_string(),
        "  Usage:".to_string(),
        "    dt                   Open interactive tree from current directory".to_string(),
        "    dt /path/to/dir      Jump directly to path (no TUI)".to_string(),
        "    dt myproject         Jump directly to bookmark 'myproject' (no TUI)".to_string(),
        "    dt -bm               List all bookmarks".to_string(),
        "    dt -bm add work      Save current directory as 'work'".to_string(),
        "    dt -bm add work /p   Save specific path as 'work'".to_string(),
        "    dt -bm remove work   Remove bookmark 'work'".to_string(),
        "    dt -v file.txt       View file in fullscreen mode".to_string(),
        "    dt -h                Show help".to_string(),
        "    dt --version         Show version".to_string(),
        "".to_string(),
        "  Behavior:".to_string(),
        "    • dt (no args) = Interactive TUI".to_string(),
        "    • dt <path/bookmark> = Direct cd (no TUI)".to_string(),
        "    • Priority: Bookmark → Path → Error".to_string(),
        "".to_string(),
        "CONFIGURATION".to_string(),
        "  dtree uses a configuration file located at:".to_string(),
        "    ~/.config/dtree/config.toml".to_string(),
        "".to_string(),
        "  On first run, this file is automatically created with default values.".to_string(),
        "  You can edit it to customize:".to_string(),
        "    • Appearance (colors, split position, icons)".to_string(),
        "    • Behavior (max file lines, show hidden files, double-click timeout)".to_string(),
        "    • Keybindings (customize keyboard shortcuts)".to_string(),
        "".to_string(),
        "  To reset to defaults: delete the config file, it will be recreated on next run.".to_string(),
        "".to_string(),
        "Press 'i' again to close this help screen".to_string(),
    ]
}
