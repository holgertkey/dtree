use ratatui::{
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    style::{Color, Modifier, Style},
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
use crate::file_icons;

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

            self.render_tree(frame, chunks[0], nav, config, show_sizes, show_files, dir_size_cache);
            self.render_file_viewer(frame, chunks[1], file_viewer, show_help, config);
        } else {
            self.tree_area_start = tree_area.x;
            self.tree_area_end = tree_area.x + tree_area.width;
            self.render_tree(frame, tree_area, nav, config, show_sizes, show_files, dir_size_cache);
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

    fn render_tree(&mut self, frame: &mut Frame, area: Rect, nav: &Navigation, config: &Config, show_sizes: bool, show_files: bool, dir_size_cache: &DirSizeCache) {
        self.tree_area_top = area.y;
        self.tree_area_height = area.height;

        let items: Vec<ListItem> = nav.flat_list.iter().map(|node| {
            let node_borrowed = node.borrow();
            let indent = "  ".repeat(node_borrowed.depth);

            // Icon with error indicator or file type icon
            let icon = if node_borrowed.has_error {
                // Error indicator always shows, regardless of icon settings
                "⚠ ".to_string()
            } else if config.appearance.show_icons {
                // Use file type icons from nerd-fonts
                let file_icon = file_icons::get_icon(&node_borrowed.path, node_borrowed.is_dir, true);
                // Fallback to arrows if icon is empty or whitespace-only
                if file_icon.trim().is_empty() {
                    if node_borrowed.is_dir {
                        if node_borrowed.is_expanded { "▼ ".to_string() } else { "▶ ".to_string() }
                    } else {
                        "  ".to_string()
                    }
                } else {
                    format!("{} ", file_icon)
                }
            } else {
                // Default arrows/markers (original behavior)
                if node_borrowed.is_dir {
                    if node_borrowed.is_expanded { "▼ ".to_string() } else { "▶ ".to_string() }
                } else {
                    "  ".to_string()
                }
            };

            // Build text with optional size column (after directory/file name)
            let text = if show_sizes {
                let size_str = if node_borrowed.is_dir {
                    // Directory size (from cache) - always show if show_sizes is enabled
                    if let Some((size, is_partial)) = dir_size_cache.get(&node_borrowed.path) {
                        format!(" [{:>7}]", DirSizeCache::format_size(size, is_partial))
                    } else if dir_size_cache.is_calculating(&node_borrowed.path) {
                        " [ calc.]".to_string()
                    } else {
                        "".to_string()
                    }
                } else if show_files {
                    // File size (from metadata) - only show if in file viewer mode (s)
                    if let Ok(metadata) = std::fs::metadata(&node_borrowed.path) {
                        format!(" [{:>7}]", DirSizeCache::format_size(metadata.len(), false))
                    } else {
                        "".to_string()
                    }
                } else {
                    "".to_string()
                };
                format!("{}{}{}{}", indent, icon, node_borrowed.name, size_str)
            } else {
                format!("{}{}{}", indent, icon, node_borrowed.name)
            };

            // Color coding: errors in configured color, directories and files use theme colors
            let style = if node_borrowed.has_error {
                let error_color = Config::parse_color(Config::get_color(&config.appearance.colors.error_color));
                Style::default().fg(error_color)
            } else if node_borrowed.is_dir {
                let dir_color = Config::parse_color(Config::get_color(&config.appearance.colors.directory_color));
                Style::default().fg(dir_color)
            } else {
                let file_color = Config::parse_color(Config::get_color(&config.appearance.colors.file_color));
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

        // Check tree cursor color settings - "dim" means no color/background, just dimming
        let tree_cursor_color_str = Config::get_color(&config.appearance.colors.tree_cursor_color);
        let tree_cursor_bg_color_str = Config::get_color(&config.appearance.colors.tree_cursor_bg_color);

        let mut highlight_style = if tree_cursor_color_str.to_lowercase() == "dim" {
            Style::default().add_modifier(Modifier::DIM)
        } else {
            let tree_cursor_color = Config::parse_color(tree_cursor_color_str);
            Style::default().fg(tree_cursor_color)
        };

        // Apply background color if not "dim"
        if tree_cursor_bg_color_str.to_lowercase() != "dim" {
            let tree_cursor_bg_color = Config::parse_color(tree_cursor_bg_color_str);
            highlight_style = highlight_style.bg(tree_cursor_bg_color);
        }

        // Apply main border color and background color
        let main_border_color = Config::parse_color(Config::get_color(&config.appearance.colors.main_border_color));
        let background_color = Config::parse_color(Config::get_color(&config.appearance.colors.background_color));

        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(main_border_color))
                .style(Style::default().bg(background_color)))
            .highlight_style(highlight_style)
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, area, &mut state);
    }

    fn render_search_bar(&self, frame: &mut Frame, area: Rect, search: &Search, config: &Config) {
        let mode_indicator = if search.fuzzy_mode { " (fuzzy)" } else { "" };
        let search_text = format!("Search{}: {}", mode_indicator, search.query);

        let selected_color = Config::parse_color(Config::get_color(&config.appearance.colors.selected_color));
        let panel_border_color = Config::parse_color(Config::get_color(&config.appearance.colors.panel_border_color));

        let title_hint = if search.fuzzy_mode {
            " Enter to search | Esc: cancel | Fuzzy mode: /query "
        } else {
            " Enter to search | Esc: cancel | Fuzzy: /query "
        };

        let paragraph = Paragraph::new(search_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title_hint)
                .border_style(Style::default().fg(panel_border_color)))
            .style(Style::default().fg(selected_color));

        frame.render_widget(paragraph, area);
    }

    fn render_search_results(&self, frame: &mut Frame, area: Rect, search: &Search, root: &TreeNodeRef, config: &Config) {
        let root_path = root.borrow().path.clone();
        let root_parent = root_path.parent().unwrap_or(&root_path);

        let file_color = Config::parse_color(Config::get_color(&config.appearance.colors.file_color));
        let highlight_color = Config::parse_color(Config::get_color(&config.appearance.colors.highlight_color));
        let panel_border_color = Config::parse_color(Config::get_color(&config.appearance.colors.panel_border_color));

        let dir_color = Config::parse_color(Config::get_color(&config.appearance.colors.directory_color));

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

        let border_style = Style::default().fg(panel_border_color);

        // Check cursor color setting - "dim" means no color highlight, just dimming
        let cursor_color_str = Config::get_color(&config.appearance.colors.cursor_color);
        let cursor_highlight_style = if cursor_color_str.to_lowercase() == "dim" {
            Style::default().add_modifier(Modifier::DIM)
        } else {
            let cursor_color = Config::parse_color(cursor_color_str);
            Style::default().fg(cursor_color).add_modifier(Modifier::BOLD)
        };

        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style))
            .highlight_style(cursor_highlight_style)
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, area, &mut state);
    }

    /// Helper method to load file with correct width for the viewer
    pub fn load_file_for_viewer(&self, file_viewer: &mut FileViewer, path: &std::path::Path, max_lines: usize, fullscreen: bool, config: &Config) -> anyhow::Result<()> {
        let enable_highlighting = config.appearance.enable_syntax_highlighting;
        let theme = &config.appearance.syntax_theme;

        if fullscreen {
            // For fullscreen, use terminal width (accounting for borders and line numbers)
            // Line numbers take ~6 chars, borders take 2, leave some margin
            let max_width = if file_viewer.show_line_numbers {
                self.terminal_width.saturating_sub(8) as usize
            } else {
                self.terminal_width.saturating_sub(2) as usize
            };
            file_viewer.load_file_with_width(path, Some(max_width), max_lines, enable_highlighting, theme)
        } else {
            // For split view, calculate available width based on split position
            let max_width = self.terminal_width
                .saturating_sub(self.split_position * self.terminal_width / 100)
                .saturating_sub(4) as usize;
            file_viewer.load_file_with_width(path, Some(max_width), max_lines, enable_highlighting, theme)
        }
    }

    fn render_file_viewer(&mut self, frame: &mut Frame, area: Rect, file_viewer: &FileViewer, show_help: bool, config: &Config) {
        // Check if we're in fullscreen mode (area == frame.area())
        let is_fullscreen = area == frame.area();

        // Reserve space for search bar if in fullscreen search mode
        let (viewer_area, search_bar_area) = if is_fullscreen && file_viewer.search_mode {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(3),
                    Constraint::Length(3),
                ])
                .split(area);
            (chunks[0], Some(chunks[1]))
        } else {
            (area, None)
        };

        self.viewer_area_start = viewer_area.x;
        self.viewer_area_top = viewer_area.y;
        self.viewer_area_height = viewer_area.height;

        // Apply main border color and background color
        let main_border_color = Config::parse_color(Config::get_color(&config.appearance.colors.main_border_color));
        let background_color = Config::parse_color(Config::get_color(&config.appearance.colors.background_color));

        let content_height = viewer_area.height.saturating_sub(2) as usize;

        let content_to_display = if show_help {
            get_help_content()
        } else {
            file_viewer.content.clone()
        };

        // Calculate visible lines (leaving space for separator and file info)
        let lines_to_show = content_height.saturating_sub(2);

        let show_numbers = is_fullscreen && file_viewer.show_line_numbers && !show_help;

        // Get highlight color for file search matches
        let file_search_highlight_color = Config::parse_color(Config::get_color(&config.appearance.colors.file_search_highlight_color));

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
                    // idx already contains scroll offset after enumerate().skip()
                    let line_idx = idx;  // 0-indexed absolute position in content
                    let line_num = line_idx + 1;  // 1-indexed for display
                    let is_match = file_viewer.line_has_match(line_idx);
                    let is_current = file_viewer.is_current_match(line_idx);
                    let is_selected = file_viewer.is_line_selected(line_idx);
                    let is_visual_cursor = file_viewer.visual_mode && file_viewer.visual_cursor == line_idx;

                    let mut spans = Vec::new();

                    // Add line numbers if enabled
                    if show_numbers {
                        let border_color = Config::parse_color(Config::get_color(&config.appearance.colors.border_color));
                        let num_style = if is_visual_cursor {
                            Style::default().fg(Color::White).bg(Color::Blue).add_modifier(Modifier::BOLD)
                        } else if is_selected {
                            Style::default().fg(Color::White).bg(Color::DarkGray)
                        } else if is_current {
                            Style::default().fg(file_search_highlight_color).add_modifier(Modifier::BOLD)
                        } else if is_match {
                            Style::default().fg(file_search_highlight_color)
                        } else {
                            Style::default().fg(border_color)
                        };
                        spans.push(Span::styled(format!("{:4} ", line_num), num_style));
                    }

                    // Add line content with search query highlighting
                    if is_match && !file_viewer.search_query.is_empty() {
                        // Highlight only the search query within the line
                        let query_lower = file_viewer.search_query.to_lowercase();

                        for span in &line.spans {
                            let text = &span.content;
                            let text_lower = text.to_lowercase();

                            if let Some(match_pos) = text_lower.find(&query_lower) {
                                // Split text into: before | match | after
                                let before = &text[..match_pos];
                                let matched = &text[match_pos..match_pos + file_viewer.search_query.len()];
                                let after = &text[match_pos + file_viewer.search_query.len()..];

                                // Apply visual selection style if needed
                                let base_style = if is_visual_cursor {
                                    span.style.bg(Color::Blue)
                                } else if is_selected {
                                    span.style.bg(Color::DarkGray)
                                } else {
                                    span.style
                                };

                                // Before match
                                if !before.is_empty() {
                                    spans.push(Span::styled(before.to_string(), base_style));
                                }

                                // Matched text
                                let match_style = if is_current {
                                    base_style.add_modifier(Modifier::REVERSED | Modifier::BOLD)
                                } else {
                                    base_style.bg(file_search_highlight_color).add_modifier(Modifier::BOLD)
                                };
                                spans.push(Span::styled(matched.to_string(), match_style));

                                // After match
                                if !after.is_empty() {
                                    spans.push(Span::styled(after.to_string(), base_style));
                                }
                            } else {
                                // No match in this span, use original style with visual selection
                                let base_style = if is_visual_cursor {
                                    span.style.bg(Color::Blue)
                                } else if is_selected {
                                    span.style.bg(Color::DarkGray)
                                } else {
                                    span.style
                                };
                                spans.push(Span::styled(span.content.clone(), base_style));
                            }
                        }
                    } else {
                        // No match or no query - use original styling with visual selection
                        for span in &line.spans {
                            let base_style = if is_visual_cursor {
                                span.style.bg(Color::Blue)
                            } else if is_selected {
                                span.style.bg(Color::DarkGray)
                            } else {
                                span.style
                            };
                            spans.push(Span::styled(span.content.clone(), base_style));
                        }
                    }

                    Line::from(spans)
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
                    // idx already contains scroll offset after enumerate().skip()
                    let line_idx = idx;  // 0-indexed absolute position in content
                    let line_num = line_idx + 1;  // 1-indexed for display
                    let is_match = file_viewer.line_has_match(line_idx);
                    let is_current = file_viewer.is_current_match(line_idx);
                    let is_selected = file_viewer.is_line_selected(line_idx);
                    let is_visual_cursor = file_viewer.visual_mode && file_viewer.visual_cursor == line_idx;

                    let mut spans = Vec::new();

                    // Add line numbers if enabled
                    if show_numbers {
                        let border_color = Config::parse_color(Config::get_color(&config.appearance.colors.border_color));
                        let num_style = if is_visual_cursor {
                            Style::default().fg(Color::White).bg(Color::Blue).add_modifier(Modifier::BOLD)
                        } else if is_selected {
                            Style::default().fg(Color::White).bg(Color::DarkGray)
                        } else if is_current {
                            Style::default().fg(file_search_highlight_color).add_modifier(Modifier::BOLD)
                        } else if is_match {
                            Style::default().fg(file_search_highlight_color)
                        } else {
                            Style::default().fg(border_color)
                        };
                        spans.push(Span::styled(format!("{:4} ", line_num), num_style));
                    }

                    // Determine base style for visual selection
                    let base_style = if is_visual_cursor {
                        Style::default().bg(Color::Blue)
                    } else if is_selected {
                        Style::default().bg(Color::DarkGray)
                    } else {
                        Style::default()
                    };

                    // Add line content with search query highlighting
                    if is_match && !file_viewer.search_query.is_empty() {
                        // Highlight only the search query within the line
                        let text = line.as_str();
                        let text_lower = text.to_lowercase();
                        let query_lower = file_viewer.search_query.to_lowercase();

                        if let Some(match_pos) = text_lower.find(&query_lower) {
                            // Split text into: before | match | after
                            let before = &text[..match_pos];
                            let matched = &text[match_pos..match_pos + file_viewer.search_query.len()];
                            let after = &text[match_pos + file_viewer.search_query.len()..];

                            // Before match
                            if !before.is_empty() {
                                spans.push(Span::styled(before.to_string(), base_style));
                            }

                            // Matched text
                            let match_style = if is_current {
                                base_style.add_modifier(Modifier::REVERSED | Modifier::BOLD)
                            } else {
                                base_style.bg(file_search_highlight_color).add_modifier(Modifier::BOLD)
                            };
                            spans.push(Span::styled(matched.to_string(), match_style));

                            // After match
                            if !after.is_empty() {
                                spans.push(Span::styled(after.to_string(), base_style));
                            }
                        } else {
                            // Shouldn't happen, but fallback to normal
                            spans.push(Span::styled(line.as_str(), base_style));
                        }
                    } else {
                        // No match or no query - normal text with visual selection style
                        spans.push(Span::styled(line.as_str(), base_style));
                    }

                    Line::from(spans)
                })
                .collect()
        };

        // Add separator and file info at the end (only if not help)
        if !show_help && !file_viewer.current_path.as_os_str().is_empty() {
            let file_info = file_viewer.format_file_info();
            let separator = "─".repeat(area.width.saturating_sub(2) as usize);

            let border_color = Config::parse_color(Config::get_color(&config.appearance.colors.border_color));

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
            // Fullscreen mode - simplified title
            let mode_indicator = if file_viewer.visual_mode {
                " [VISUAL MODE]"
            } else if file_viewer.tail_mode {
                " [TAIL MODE]"
            } else if file_viewer.total_lines.is_some() && file_viewer.total_lines.unwrap() > file_viewer.content.len() {
                " [HEAD MODE]"
            } else {
                ""
            };

            // Add search match info if there are results or in search mode
            let search_info = if !file_viewer.search_results.is_empty() {
                format!(" | {} | n/N: next/prev ", file_viewer.get_match_info())
            } else if file_viewer.search_mode && !file_viewer.search_query.is_empty() {
                " | No matches ".to_string()
            } else {
                String::new()
            };

            // Add hints for toggles (hide in visual mode)
            let hints = if file_viewer.visual_mode {
                " - j/k: select | y: copy | Esc: cancel"
            } else {
                let line_numbers_hint = if file_viewer.show_line_numbers {
                    " | l: hide lines"
                } else {
                    " | l: show lines"
                };

                let wrap_hint = if file_viewer.wrap_lines {
                    " | w: truncate"
                } else {
                    " | w: wrap"
                };

                &format!(" - V: visual | /: search | j/k: scroll | Ctrl+j/k: next/prev file{}{} | q: back | Esc: exit", line_numbers_hint, wrap_hint)
            };

            format!(" File Viewer (Fullscreen{}{}){}{}",
                mode_indicator, hints, search_info, scroll_info)
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
                .title(title)
                .border_style(Style::default().fg(main_border_color))
                .style(Style::default().bg(background_color)));

        frame.render_widget(paragraph, viewer_area);

        // Render file search bar if in search mode
        if let Some(search_area) = search_bar_area {
            self.render_file_search_bar(frame, search_area, file_viewer, config);
        }
    }

    fn render_file_search_bar(&self, frame: &mut Frame, area: Rect, file_viewer: &FileViewer, config: &Config) {
        let search_text = format!("Search: {}█", file_viewer.search_query);

        let selected_color = Config::parse_color(Config::get_color(&config.appearance.colors.selected_color));
        let panel_border_color = Config::parse_color(Config::get_color(&config.appearance.colors.panel_border_color));

        let title_hint = " Enter to search | n: next | N: prev | Esc: cancel ";

        let paragraph = Paragraph::new(search_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title_hint)
                .border_style(Style::default().fg(panel_border_color)))
            .style(Style::default().fg(selected_color));

        frame.render_widget(paragraph, area);
    }

    fn render_bookmarks_panel(&self, frame: &mut Frame, area: Rect, bookmarks: &Bookmarks, config: &Config) {
        let border_color = Config::parse_color(Config::get_color(&config.appearance.colors.border_color));
        let selected_color = Config::parse_color(Config::get_color(&config.appearance.colors.selected_color));
        let file_color = Config::parse_color(Config::get_color(&config.appearance.colors.file_color));
        let panel_border_color = Config::parse_color(Config::get_color(&config.appearance.colors.panel_border_color));

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
                        .title(count_text)
                        .border_style(Style::default().fg(panel_border_color)));

                frame.render_widget(list, list_area);
            }

            // Render input bar at the bottom
            let input_text = format!("Bookmark name: {}█", bookmarks.get_input());
            let title = " Create Bookmark (Enter: save | Esc: cancel | Ctrl+j/k/↑↓: scroll list) ";

            let paragraph = Paragraph::new(input_text)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(panel_border_color)))
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
                let error_color = Config::parse_color(Config::get_color(&config.appearance.colors.error_color));
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

                // Check cursor color setting - "dim" means no color highlight, just dimming
                let cursor_color_str = Config::get_color(&config.appearance.colors.cursor_color);
                let cursor_highlight_style = if cursor_color_str.to_lowercase() == "dim" {
                    Style::default().add_modifier(Modifier::DIM)
                } else {
                    let cursor_color = Config::parse_color(cursor_color_str);
                    Style::default().fg(cursor_color).add_modifier(Modifier::BOLD)
                };

                let list = List::new(items)
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .title(hint)
                        .border_style(Style::default().fg(panel_border_color)))
                    .highlight_style(cursor_highlight_style)
                    .highlight_symbol(">> ");

                frame.render_stateful_widget(list, area, &mut state);
            }
        }
    }
}

/// Load help content from HELP.txt file (embedded at compile time)
pub fn get_help_content() -> Vec<String> {
    // Embed HELP.txt at compile time using include_str!
    // This is more reliable than runtime file I/O
    const HELP_TEXT: &str = include_str!("../HELP.txt");

    // Split by lines and convert to Vec<String>
    HELP_TEXT.lines().map(|line| line.to_string()).collect()
}
