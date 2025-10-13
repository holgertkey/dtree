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
use crate::dir_size::DirSizeCache;

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
    pub terminal_height: u16,
    pub split_position: u16,
    pub tree_scroll_offset: usize,
    // Bottom panel (search/bookmarks) properties
    pub bottom_panel_split_position: u16, // Percentage from top (default 70)
    pub bottom_panel_top: u16,
    pub bottom_panel_height: u16,
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
            terminal_height: 0,
            split_position: 50,
            tree_scroll_offset: 0,
            bottom_panel_split_position: 70,
            bottom_panel_top: 0,
            bottom_panel_height: 0,
        }
    }

    /// Adjust horizontal split position (20-80% range)
    pub fn adjust_split(&mut self, position: u16) {
        self.split_position = position.clamp(20, 80);
    }

    /// Adjust vertical split position for bottom panel (30-90% range)
    pub fn adjust_bottom_split(&mut self, position: u16) {
        self.bottom_panel_split_position = position.clamp(30, 90);
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
        show_sizes: bool,
        dir_size_cache: &DirSizeCache,
    ) {
        self.terminal_width = frame.area().width;
        self.terminal_height = frame.area().height;
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

        // If showing search results or bookmarks, split vertically with dynamic position
        let (tree_area, bottom_panel_area) = if search.show_results || bookmarks.is_selecting || bookmarks.is_creating {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(self.bottom_panel_split_position),
                    Constraint::Percentage(100 - self.bottom_panel_split_position),
                ])
                .split(content_area);

            // Save bottom panel coordinates for mouse handling
            self.bottom_panel_top = chunks[1].y;
            self.bottom_panel_height = chunks[1].height;

            (chunks[0], Some(chunks[1]))
        } else {
            // Reset bottom panel coordinates when not visible
            self.bottom_panel_top = 0;
            self.bottom_panel_height = 0;
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

            self.render_tree(frame, chunks[0], nav, config, show_sizes, dir_size_cache);
            self.render_file_viewer(frame, chunks[1], file_viewer, show_help, config);
        } else {
            self.tree_area_start = tree_area.x;
            self.tree_area_end = tree_area.x + tree_area.width;
            self.render_tree(frame, tree_area, nav, config, show_sizes, dir_size_cache);
        }

        // Render bottom panel - bookmarks take priority over search results
        if let Some(area) = bottom_panel_area {
            if bookmarks.is_selecting || bookmarks.is_creating {
                self.render_bookmarks_panel(frame, area, bookmarks, config);
            } else if search.show_results {
                self.render_search_results(frame, area, search, &nav.root, config);
            }
        }

        // Render search bar if in input mode
        if let Some(area) = search_bar_area {
            self.render_search_bar(frame, area, search, config);
        }
    }

    fn render_tree(&mut self, frame: &mut Frame, area: Rect, nav: &Navigation, config: &Config, show_sizes: bool, dir_size_cache: &DirSizeCache) {
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

            // Build text with optional size column (after directory name)
            let text = if show_sizes && node_borrowed.is_dir {
                let size_str = if let Some(size) = dir_size_cache.get(&node_borrowed.path) {
                    format!(" [{:>7}]", DirSizeCache::format_size(size))
                } else if dir_size_cache.is_calculating(&node_borrowed.path) {
                    " [ calc.]".to_string()
                } else {
                    "".to_string()
                };
                format!("{}{}{}{}", indent, icon, node_borrowed.name, size_str)
            } else {
                format!("{}{}{}", indent, icon, node_borrowed.name)
            };

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

        let title = if show_sizes {
            " Directory Tree (↑↓/jk: navigate | Enter: go in | q: cd & exit | Esc: exit | z: hide sizes | /: search | i: help) "
        } else {
            " Directory Tree (↑↓/jk: navigate | Enter: go in | q: cd & exit | Esc: exit | z: show sizes | /: search | i: help) "
        };

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
        let mode_indicator = if search.fuzzy_mode { " (fuzzy)" } else { "" };
        let search_text = format!("Search{}: {}", mode_indicator, search.query);

        let selected_color = Config::parse_color(&config.appearance.colors.selected_color);

        let title_hint = if search.fuzzy_mode {
            " Enter to search | Esc: cancel | Fuzzy mode: /query "
        } else {
            " Enter to search | Esc: cancel | Fuzzy: /query "
        };

        let paragraph = Paragraph::new(search_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title_hint))
            .style(Style::default().fg(selected_color));

        frame.render_widget(paragraph, area);
    }

    fn render_search_results(&self, frame: &mut Frame, area: Rect, search: &Search, root: &TreeNodeRef, config: &Config) {
        let root_path = root.borrow().path.clone();
        let root_parent = root_path.parent().unwrap_or(&root_path);

        let file_color = Config::parse_color(&config.appearance.colors.file_color);
        let selected_color = Config::parse_color(&config.appearance.colors.selected_color);
        let highlight_color = Config::parse_color(&config.appearance.colors.highlight_color);

        let dir_color = Config::parse_color(&config.appearance.colors.directory_color);

        let items: Vec<ListItem> = search.results.iter().map(|result| {
            let display_path = result.path.strip_prefix(root_parent)
                .unwrap_or(&result.path)
                .display()
                .to_string();

            let base_color = if result.is_dir { dir_color } else { file_color };

            // In fuzzy mode with match indices, highlight matching characters
            if search.fuzzy_mode && result.match_indices.is_some() {
                let mut spans = Vec::new();
                let chars: Vec<char> = display_path.chars().collect();
                let indices = result.match_indices.as_ref().unwrap();
                let mut last_idx = 0;

                for &match_idx in indices {
                    // Add text before the match
                    if match_idx > last_idx {
                        let text: String = chars[last_idx..match_idx].iter().collect();
                        spans.push(Span::styled(text, Style::default().fg(base_color)));
                    }

                    // Add highlighted character
                    if match_idx < chars.len() {
                        let text: String = chars[match_idx..match_idx+1].iter().collect();
                        spans.push(Span::styled(text, Style::default().fg(highlight_color).add_modifier(Modifier::BOLD)));
                    }

                    last_idx = match_idx + 1;
                }

                // Add remaining text after last match
                if last_idx < chars.len() {
                    let text: String = chars[last_idx..].iter().collect();
                    spans.push(Span::styled(text, Style::default().fg(base_color)));
                }

                // Add score at the end
                if let Some(score) = result.score {
                    spans.push(Span::styled(format!(" [{}]", score), Style::default().fg(base_color)));
                }

                ListItem::new(Line::from(spans))
            } else {
                // Normal mode or no match indices - just display path with optional score
                let display_text = if search.fuzzy_mode && result.score.is_some() {
                    format!("{} [{}]", display_path, result.score.unwrap())
                } else {
                    display_path
                };

                ListItem::new(display_text).style(Style::default().fg(base_color))
            }
        }).collect();

        let mut state = ListState::default();
        state.select(Some(search.selected));

        // Show search status in title
        let title = if search.is_searching {
            format!(" Search: {} found | Scanning... {} dirs | Esc: cancel ",
                search.results.len(), search.scanned_count)
        } else {
            format!(" Search Results: {} found | Enter: select | Tab: focus | Esc: close ",
                search.results.len())
        };

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
            format!(" File Viewer (Fullscreen - {} | Ctrl+j/k: scroll | q: back | Esc: exit){} ", line_num_hint, scroll_info)
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

    fn render_bookmarks_panel(&self, frame: &mut Frame, area: Rect, bookmarks: &Bookmarks, config: &Config) {
        let border_color = Config::parse_color(&config.appearance.colors.border_color);
        let selected_color = Config::parse_color(&config.appearance.colors.selected_color);
        let highlight_color = Config::parse_color(&config.appearance.colors.highlight_color);
        let file_color = Config::parse_color(&config.appearance.colors.file_color);

        if bookmarks.is_creating {
            // Creation mode - bookmark list + input bar
            // Split area: top for list, bottom for input (3 lines)
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(3),
                    Constraint::Length(3),
                ])
                .split(area);

            let list_area = chunks[0];
            let input_area = chunks[1];

            // Render bookmark list first
            let all_bookmarks = bookmarks.list();
            if !all_bookmarks.is_empty() {
                let items: Vec<ListItem> = all_bookmarks.iter().skip(bookmarks.scroll_offset).map(|bookmark| {
                    let name = bookmark.name.as_deref().unwrap_or("(unnamed)");
                    let path_str = bookmark.path.display().to_string();

                    let text = format!("{:<12} → {:<20} ({})", bookmark.key, name, path_str);
                    ListItem::new(text).style(Style::default().fg(file_color))
                }).collect();

                let count_text = if all_bookmarks.len() > 0 {
                    format!(" Existing Bookmarks ({}) ", all_bookmarks.len())
                } else {
                    " Existing Bookmarks ".to_string()
                };

                let list = List::new(items)
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .title(count_text));

                frame.render_widget(list, list_area);
            }

            // Render input bar at the bottom
            let input_text = format!("Bookmark name: {}█", bookmarks.get_input());
            let title = " Create Bookmark (Enter: save | Esc: cancel | Ctrl+j/k/↑↓: scroll list) ";

            let paragraph = Paragraph::new(input_text)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(title))
                .style(Style::default().fg(selected_color).add_modifier(Modifier::BOLD));

            frame.render_widget(paragraph, input_area);
        } else {
            // Selection mode - list with navigation
            let filtered = bookmarks.get_filtered_bookmarks();

            if filtered.is_empty() {
                // No bookmarks - show message
                let title = " Bookmarks (Press 'm' to create | Esc: close) ";
                let message = if bookmarks.filter_mode {
                    format!("No bookmarks match filter: '{}'", bookmarks.get_input())
                } else {
                    "No bookmarks saved yet".to_string()
                };

                let paragraph = Paragraph::new(message)
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .title(title)
                        .border_style(Style::default().fg(border_color)))
                    .style(Style::default().fg(border_color));

                frame.render_widget(paragraph, area);
            } else {
                // Has bookmarks - show list with navigation
                let error_color = Config::parse_color(&config.appearance.colors.error_color);
                let items: Vec<ListItem> = filtered.iter().enumerate().map(|(idx, bookmark)| {
                    let name = bookmark.name.as_deref().unwrap_or("(unnamed)");
                    let path_str = bookmark.path.display().to_string();

                    // Check if this bookmark is marked for deletion
                    let is_marked = bookmarks.pending_deletion_index == Some(idx);
                    let prefix = if is_marked { "[DEL] " } else { "" };
                    let text = format!("{}{:<12} → {:<20} ({})", prefix, bookmark.key, name, path_str);

                    // Use error color for marked bookmarks
                    let style = if is_marked {
                        Style::default().fg(error_color)
                    } else {
                        Style::default().fg(file_color)
                    };

                    ListItem::new(text).style(style)
                }).collect();

                let mut state = ListState::default();
                state.select(Some(bookmarks.selected_index));

                let mode_hint = if bookmarks.filter_mode {
                    format!("Filter: {}", bookmarks.get_input())
                } else {
                    format!("{}/{}", bookmarks.selected_index + 1, filtered.len())
                };

                let deletion_hint = if bookmarks.is_marked_for_deletion() {
                    " | d: confirm delete"
                } else {
                    " | d: delete"
                };

                let hint = if bookmarks.filter_mode {
                    format!(" {} | Tab: nav | Enter: select | Esc: cancel ", mode_hint)
                } else {
                    format!(" Bookmarks: {} | ↑↓/jk: move{} | Tab: filter | Enter: select | Esc: cancel ", mode_hint, deletion_hint)
                };

                let list = List::new(items)
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .title(hint)
                        .border_style(Style::default().fg(selected_color)))
                    .highlight_style(Style::default()
                        .fg(highlight_color)
                        .add_modifier(Modifier::BOLD))
                    .highlight_symbol(">> ");

                frame.render_stateful_widget(list, area, &mut state);
            }
        }
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
        "  z              Toggle directory size display (shows calculated sizes)".to_string(),
        "  i              Show/hide this help screen".to_string(),
        "".to_string(),
        "DIRECTORY SIZE DISPLAY (press 'z' to toggle)".to_string(),
        "  When enabled:".to_string(),
        "    • Shows total size for each directory next to its name".to_string(),
        "    • Sizes are calculated asynchronously in the background".to_string(),
        "    • Shows 'calc.' while calculation is in progress".to_string(),
        "    • Format: K (kilobytes), M (megabytes), G (gigabytes), T (terabytes)".to_string(),
        "    • Sizes include all files and subdirectories recursively".to_string(),
        "    • Results are cached until you toggle off or navigate away".to_string(),
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
        "  • Resizable panel: drag top border with mouse to adjust height".to_string(),
        "  • Cyan border indicates which panel has focus".to_string(),
        "".to_string(),
        "  Search modes:".to_string(),
        "  • Normal search (query):   Case-insensitive substring matching".to_string(),
        "                             Example: type 'search' to find 'search.rs'".to_string(),
        "".to_string(),
        "  • Fuzzy search (/query):   Intelligent fuzzy matching with scoring".to_string(),
        "                             Start query with '/' to activate fuzzy mode".to_string(),
        "                             Example: type '/srch' to find 'search.rs'".to_string(),
        "                             Results show score [95] and highlight matches".to_string(),
        "                             Best matches ranked first by relevance score".to_string(),
        "".to_string(),
        "BOOKMARKS".to_string(),
        "  Interactive mode (inside dtree):".to_string(),
        "    m            Enter bookmark creation mode".to_string(),
        "    '            Open bookmark selection menu (tick/apostrophe)".to_string(),
        "".to_string(),
        "  Bookmark Creation (after pressing 'm'):".to_string(),
        "    • Bottom panel shows: input bar + list of existing bookmarks".to_string(),
        "    • Type bookmark name (multi-character names supported)".to_string(),
        "    • Examples: work, project-123, my_home".to_string(),
        "    • Ctrl+j/k (or Ctrl+↑↓) scrolls through existing bookmarks list".to_string(),
        "    • Enter to save, Esc to cancel".to_string(),
        "    • NOTE: Bookmarks save directories only (if cursor on file, saves parent dir)".to_string(),
        "".to_string(),
        "  Bookmark Selection (after pressing '''):".to_string(),
        "    • Bottom panel shows all saved bookmarks with paths".to_string(),
        "    • Two modes: Navigation (default) and Filter".to_string(),
        "    • Tab switches between navigation and filter modes".to_string(),
        "".to_string(),
        "    Navigation mode:".to_string(),
        "      ↑↓ / jk     Move selection up/down".to_string(),
        "      d           Delete bookmark (press once to mark, twice to confirm)".to_string(),
        "      Enter       Jump to selected bookmark".to_string(),
        "      Tab         Switch to filter mode".to_string(),
        "".to_string(),
        "    Filter mode:".to_string(),
        "      Type text   Filter bookmarks by name or path".to_string(),
        "      Tab         Switch to navigation mode (keeps filter)".to_string(),
        "      ↑↓ / jk     Navigate through filtered results (in nav mode)".to_string(),
        "      Enter       Jump to selected bookmark".to_string(),
        "      Esc         Close bookmarks panel".to_string(),
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
        "  # Store current directory before navigation".to_string(),
        "  local prev_dir=\"$PWD\"".to_string(),
        "".to_string(),
        "  # Handle special case: dt - (return to previous directory)".to_string(),
        "  if [ \"$1\" = \"-\" ]; then".to_string(),
        "    if [ -n \"$DTREE_PREV_DIR\" ] && [ -d \"$DTREE_PREV_DIR\" ]; then".to_string(),
        "      cd \"$DTREE_PREV_DIR\" || return".to_string(),
        "      export DTREE_PREV_DIR=\"$prev_dir\"".to_string(),
        "    else".to_string(),
        "      echo \"dt: no previous directory\" >&2".to_string(),
        "      return 1".to_string(),
        "    fi".to_string(),
        "    return".to_string(),
        "  fi".to_string(),
        "".to_string(),
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
        "    # Save previous directory for dt -".to_string(),
        "    export DTREE_PREV_DIR=\"$prev_dir\"".to_string(),
        "  fi".to_string(),
        "}".to_string(),
        "".to_string(),
        "  Usage:".to_string(),
        "    dt                   Open interactive tree from current directory".to_string(),
        "    dt /path/to/dir      Jump directly to path (no TUI)".to_string(),
        "    dt myproject         Jump directly to bookmark 'myproject' (no TUI)".to_string(),
        "    dt -                 Return to previous directory (like cd -)".to_string(),
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
