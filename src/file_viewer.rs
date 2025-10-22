use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::unix::fs::PermissionsExt;
use anyhow::Result;
use unicode_truncate::UnicodeTruncateStr;
use unicode_width::UnicodeWidthStr;
use ratatui::text::{Line, Span};
use ratatui::style::{Style, Color};
use once_cell::sync::Lazy;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;
use syntect::easy::HighlightLines;

/// Lazy-loaded syntax set (loaded once on first use)
static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);

/// Lazy-loaded theme set (loaded once on first use)
static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

/// File viewer state and logic for displaying file contents
pub struct FileViewer {
    pub content: Vec<String>,
    pub highlighted_content: Vec<Line<'static>>,
    pub scroll: usize,
    pub current_path: PathBuf,
    pub current_size: u64,
    pub current_permissions: u32,
    pub show_line_numbers: bool,
    pub wrap_lines: bool,  // true = wrap long lines, false = truncate
    pub syntax_name: Option<String>,
    pub is_binary: bool,
    pub tail_mode: bool,  // true = showing last N lines, false = showing first N lines
    pub total_lines: Option<usize>,  // total lines in file (if known)

    // Search functionality
    pub search_mode: bool,
    pub search_query: String,
    pub search_results: Vec<usize>,  // Line numbers with matches (0-indexed)
    pub current_match: usize,  // Current match index in search_results

    // Visual selection mode
    pub visual_mode: bool,
    pub visual_start: Option<usize>,  // Start line of selection (0-indexed)
    pub visual_cursor: usize,  // Current cursor position in visual mode (0-indexed)
}

impl Default for FileViewer {
    fn default() -> Self {
        Self::new()
    }
}

impl FileViewer {
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
            highlighted_content: Vec::new(),
            scroll: 0,
            current_path: PathBuf::new(),
            current_size: 0,
            current_permissions: 0,
            show_line_numbers: false,
            wrap_lines: true,  // Default to wrapping enabled
            syntax_name: None,
            is_binary: false,
            tail_mode: false,
            total_lines: None,
            search_mode: false,
            search_query: String::new(),
            search_results: Vec::new(),
            current_match: 0,
            visual_mode: false,
            visual_start: None,
            visual_cursor: 0,
        }
    }

    /// Toggle line numbers display
    pub fn toggle_line_numbers(&mut self) {
        self.show_line_numbers = !self.show_line_numbers;
    }

    /// Toggle line wrapping
    pub fn toggle_wrap(&mut self) {
        self.wrap_lines = !self.wrap_lines;
    }

    /// Read last N lines from a file (for tail mode)
    fn read_tail_lines(path: &Path, max_lines: usize) -> Result<(Vec<String>, usize)> {
        use std::io::{Seek, SeekFrom};

        let mut file = File::open(path)?;
        let file_size = file.metadata()?.len();

        // If file is small, just read all lines
        if file_size < 1024 * 1024 {  // < 1MB
            let reader = BufReader::new(file);
            let all_lines: Vec<String> = reader.lines().map_while(Result::ok).collect();
            let total = all_lines.len();

            if total <= max_lines {
                return Ok((all_lines, total));
            }

            let start_idx = total.saturating_sub(max_lines);
            return Ok((all_lines[start_idx..].to_vec(), total));
        }

        // For large files: seek backward from end
        // Strategy: read chunks backwards until we have enough lines
        const CHUNK_SIZE: u64 = 64 * 1024;  // 64KB chunks
        let mut buffer = Vec::new();
        let mut current_pos = file_size;

        loop {
            // Calculate chunk position
            let chunk_start = current_pos.saturating_sub(CHUNK_SIZE);
            let chunk_size = (current_pos - chunk_start) as usize;

            // Seek to chunk start
            file.seek(SeekFrom::Start(chunk_start))?;

            // Read chunk
            let mut chunk = vec![0u8; chunk_size];
            std::io::Read::read_exact(&mut file, &mut chunk)?;

            // Prepend to buffer
            chunk.append(&mut buffer);
            buffer = chunk;

            // Count lines in buffer
            let lines_found = buffer.iter().filter(|&&b| b == b'\n').count();

            // If we have enough lines or reached start, stop
            if lines_found >= max_lines || chunk_start == 0 {
                break;
            }

            current_pos = chunk_start;
        }

        // Convert buffer to string and split into lines
        let text = String::from_utf8_lossy(&buffer);
        let all_lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
        let total = all_lines.len();

        // Return last max_lines
        if total <= max_lines {
            Ok((all_lines, total))
        } else {
            let start_idx = total.saturating_sub(max_lines);
            Ok((all_lines[start_idx..].to_vec(), total))
        }
    }

    /// Load file content with specified max width and max lines
    pub fn load_file_with_width(&mut self, path: &Path, max_width: Option<usize>, max_lines: usize, enable_syntax_highlighting: bool, syntax_theme: &str) -> Result<()> {
        const DEFAULT_MAX_WIDTH: usize = 10000; // Very large default to avoid truncation

        let max_width = max_width.unwrap_or(DEFAULT_MAX_WIDTH);

        self.content.clear();
        self.highlighted_content.clear();
        self.scroll = 0;
        self.current_path = path.to_path_buf();
        self.current_size = 0;
        self.current_permissions = 0;
        self.syntax_name = None;
        self.is_binary = false;
        // Note: tail_mode is NOT reset here - it persists across reloads
        self.total_lines = None;

        // Check if this is a file
        if !path.is_file() {
            if path.is_dir() {
                self.content.push("[Directory - use arrow keys to navigate]".to_string());
            } else if path.is_symlink() {
                self.content.push("[Symbolic link]".to_string());
            } else {
                self.content.push("[Not a regular file]".to_string());
            }
            return Ok(());
        }

        // Get file metadata
        match std::fs::metadata(path) {
            Ok(metadata) => {
                self.current_size = metadata.len();
                self.current_permissions = metadata.permissions().mode();
            }
            Err(e) => {
                self.content.push(format!("[Cannot read metadata: {}]", e));
                return Ok(());
            }
        }

        // Check if file is binary before trying to read it as text
        if Self::is_binary_file(path) {
            self.is_binary = true;
            self.load_binary_info(path);
            return Ok(());
        }

        // Read file content based on mode (head or tail)
        let (raw_lines, total_lines) = if self.tail_mode {
            // Tail mode: read last N lines
            match Self::read_tail_lines(path, max_lines) {
                Ok(result) => result,
                Err(e) => {
                    self.content.push(format!("[Error reading file: {}]", e));
                    return Ok(());
                }
            }
        } else {
            // Head mode: read first N lines
            let file = match File::open(path) {
                Ok(f) => f,
                Err(e) => {
                    self.content.push(format!("[Error: {}]", e));
                    return Ok(());
                }
            };

            let reader = BufReader::new(file);
            let mut lines = Vec::new();
            let mut line_count = 0;
            let mut total = 0;

            for line in reader.lines() {
                total += 1;

                if line_count >= max_lines {
                    // Continue counting total lines even after truncation
                    continue;
                }

                match line {
                    Ok(content) => {
                        lines.push(content);
                        line_count += 1;
                    }
                    Err(e) => {
                        // Possibly binary file or encoding error
                        self.content.clear();
                        self.content.push(format!("[Binary file or encoding error: {}]", e));
                        return Ok(());
                    }
                }
            }

            (lines, total)
        };

        // Store total lines for UI display
        self.total_lines = Some(total_lines);

        // Process lines: replace tabs and wrap/truncate based on settings
        for content in raw_lines {
            // Replace tabs with spaces (4 spaces per tab)
            let content_no_tabs = content.replace('\t', "    ");

            if self.wrap_lines {
                // Wrap long lines
                let wrapped_lines = Self::wrap_line(&content_no_tabs, max_width);
                for wrapped in wrapped_lines {
                    self.content.push(wrapped);
                }
            } else {
                // Don't truncate - keep full line content for copying
                self.content.push(content_no_tabs);
            }
        }

        // Add truncation indicator if needed
        if !self.tail_mode && total_lines > max_lines {
            self.content.push(format!("\n[... truncated, showing first {} of {} lines. Press End to see tail ...]", max_lines, total_lines));
        } else if self.tail_mode && total_lines > max_lines {
            self.content.insert(0, format!("[... showing last {} of {} lines. Press Home to see head ...]", max_lines, total_lines));
        }

        if self.content.is_empty() {
            self.content.push("[Empty file]".to_string());
        }

        // Apply syntax highlighting if enabled
        if enable_syntax_highlighting && !self.content.is_empty() {
            self.apply_syntax_highlighting(syntax_theme);
        }

        Ok(())
    }

    /// Apply syntax highlighting to content
    fn apply_syntax_highlighting(&mut self, theme_name: &str) {
        // Detect syntax based on file extension
        let syntax = SYNTAX_SET
            .find_syntax_for_file(&self.current_path)
            .ok()
            .flatten()
            .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

        self.syntax_name = Some(syntax.name.clone());

        // Get theme
        let theme = THEME_SET.themes.get(theme_name)
            .unwrap_or_else(|| THEME_SET.themes.get("base16-ocean.dark").unwrap());

        // Highlight lines
        let mut highlighter = HighlightLines::new(syntax, theme);

        for line_text in &self.content {
            let highlighted = highlighter.highlight_line(line_text, &SYNTAX_SET);

            match highlighted {
                Ok(ranges) => {
                    let spans: Vec<Span> = ranges
                        .iter()
                        .map(|(style, text)| {
                            let fg_color = Self::syntect_color_to_ratatui(style.foreground);
                            Span::styled(text.to_string(), Style::default().fg(fg_color))
                        })
                        .collect();

                    self.highlighted_content.push(Line::from(spans));
                }
                Err(_) => {
                    // Fallback to plain text
                    self.highlighted_content.push(Line::from(line_text.clone()));
                }
            }
        }
    }

    /// Convert syntect color to ratatui color
    fn syntect_color_to_ratatui(color: syntect::highlighting::Color) -> Color {
        Color::Rgb(color.r, color.g, color.b)
    }

    /// Wrap a line to max_width, returning a vector of wrapped lines
    fn wrap_line(line: &str, max_width: usize) -> Vec<String> {
        if max_width == 0 {
            return vec![line.to_string()];
        }

        // If line fits, return it as-is
        let line_width = line.width();
        if line_width <= max_width {
            return vec![line.to_string()];
        }

        let mut result = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0;

        for word in line.split_whitespace() {
            let word_width = word.width();

            // If this is the first word in the line
            if current_line.is_empty() {
                // If word is longer than max_width, we must break it
                if word_width > max_width {
                    let mut remaining = word;
                    while !remaining.is_empty() {
                        let (chunk, byte_offset) = remaining.unicode_truncate(max_width);
                        result.push(chunk.to_string());
                        remaining = &remaining[byte_offset..];
                    }
                } else {
                    current_line.push_str(word);
                    current_width = word_width;
                }
            } else {
                // Check if adding space + word would exceed max_width
                let space_width = 1;
                let new_width = current_width + space_width + word_width;

                if new_width <= max_width {
                    // Add space and word to current line
                    current_line.push(' ');
                    current_line.push_str(word);
                    current_width = new_width;
                } else {
                    // Save current line and start new one
                    result.push(current_line.clone());
                    current_line.clear();

                    // If word is longer than max_width, break it
                    if word_width > max_width {
                        let mut remaining = word;
                        while !remaining.is_empty() {
                            let remaining_width = remaining.width();
                            if remaining_width <= max_width {
                                current_line = remaining.to_string();
                                current_width = remaining_width;
                                break;
                            } else {
                                let (chunk, byte_offset) = remaining.unicode_truncate(max_width);
                                result.push(chunk.to_string());
                                remaining = &remaining[byte_offset..];
                            }
                        }
                    } else {
                        current_line.push_str(word);
                        current_width = word_width;
                    }
                }
            }
        }

        // Don't forget the last line
        if !current_line.is_empty() {
            result.push(current_line);
        }

        // If nothing was added (e.g., empty line), return empty string
        if result.is_empty() {
            result.push(String::new());
        }

        result
    }

    /// Scroll down in file content
    pub fn scroll_down(&mut self, max_visible_lines: usize) {
        let max_scroll = self.content.len().saturating_sub(max_visible_lines);
        if self.scroll < max_scroll {
            self.scroll += 1;
        }
    }

    /// Scroll down by one line (simplified version)
    pub fn scroll_down_simple(&mut self) {
        if self.scroll < self.content.len().saturating_sub(1) {
            self.scroll += 1;
        }
    }

    /// Scroll up in file content
    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    /// Reset scroll position (jump to top)
    pub fn reset_scroll(&mut self) {
        self.scroll = 0;
    }

    /// Scroll up by page (visible height)
    pub fn scroll_page_up(&mut self, visible_height: usize) {
        self.scroll = self.scroll.saturating_sub(visible_height);
    }

    /// Scroll down by page (visible height)
    pub fn scroll_page_down(&mut self, visible_height: usize, max_visible_lines: usize) {
        let max_scroll = self.content.len().saturating_sub(max_visible_lines);
        self.scroll = (self.scroll + visible_height).min(max_scroll);
    }

    /// Jump to end of file
    pub fn scroll_to_end(&mut self, visible_height: usize) {
        self.scroll = self.content.len().saturating_sub(visible_height);
    }

    /// Load custom content (e.g., help text)
    pub fn load_content(&mut self, content: Vec<String>) {
        self.content = content;
        self.highlighted_content.clear();
        self.scroll = 0;
        self.current_path = PathBuf::new();
        self.current_size = 0;
        self.current_permissions = 0;
        self.syntax_name = None;
        self.is_binary = false;
        self.tail_mode = false;
        self.total_lines = None;
    }

    /// Switch to tail mode (show last N lines)
    pub fn enable_tail_mode(&mut self) {
        self.tail_mode = true;
    }

    /// Switch to head mode (show first N lines)
    pub fn enable_head_mode(&mut self) {
        self.tail_mode = false;
    }

    /// Check if file can use tail mode (is a text file and has path set)
    pub fn can_use_tail_mode(&self) -> bool {
        !self.is_binary && !self.current_path.as_os_str().is_empty()
    }

    /// Check if a file is binary by looking for NULL bytes in the first 8KB
    pub fn is_binary_file(path: &Path) -> bool {
        use std::io::Read;

        let file = match File::open(path) {
            Ok(f) => f,
            Err(_) => return false,
        };

        let mut reader = BufReader::new(file);
        let mut buffer = [0u8; 8192]; // Check first 8KB

        match reader.read(&mut buffer) {
            Ok(n) if n > 0 => {
                // Check for NULL bytes (indicator of binary data)
                buffer[..n].contains(&0)
            }
            _ => false,
        }
    }

    /// Load informational message for binary files
    fn load_binary_info(&mut self, path: &Path) {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown");

        let size_str = format_file_size(self.current_size);
        let perms_str = format_permissions(self.current_permissions);

        // Try to detect file type using file extension
        let file_type = Self::guess_binary_type(path);

        self.content = vec![
            "".to_string(),
            "╔══════════════════════════════════════════════════════════════════╗".to_string(),
            "║                         BINARY FILE                              ║".to_string(),
            "╚══════════════════════════════════════════════════════════════════╝".to_string(),
            "".to_string(),
            format!("  File: {}", file_name),
            format!("  Size: {} ({} bytes)", size_str, self.current_size),
            format!("  Type: {}", file_type),
            format!("  Permissions: {}", perms_str),
            "".to_string(),
            "  This is a binary file and cannot be displayed as text.".to_string(),
            "".to_string(),
            "  Available Actions:".to_string(),
            "    e  -  Open in hex editor (configured in config.toml)".to_string(),
            "    o  -  Open in file manager".to_string(),
            "    c  -  Copy path to clipboard".to_string(),
            "    q  -  Return to tree view".to_string(),
            "".to_string(),
            "  Tip: Configure your preferred hex editor in ~/.config/dtree/config.toml".to_string(),
            "       Default: hexyl (install with: cargo install hexyl)".to_string(),
            "".to_string(),
        ];
    }

    /// Guess binary file type based on extension
    fn guess_binary_type(path: &Path) -> String {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            // Executables
            "exe" | "dll" | "so" | "dylib" | "bin" => "Executable / Library".to_string(),
            // Archives
            "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => "Archive".to_string(),
            // Images
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "webp" => "Image".to_string(),
            // Video
            "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" => "Video".to_string(),
            // Audio
            "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" => "Audio".to_string(),
            // Documents
            "pdf" => "PDF Document".to_string(),
            "doc" | "docx" => "Word Document".to_string(),
            "xls" | "xlsx" => "Excel Spreadsheet".to_string(),
            "ppt" | "pptx" => "PowerPoint Presentation".to_string(),
            // Database
            "db" | "sqlite" | "sqlite3" => "Database".to_string(),
            // Object files
            "o" | "a" | "lib" => "Object / Library File".to_string(),
            // ISO images
            "iso" | "img" => "Disk Image".to_string(),
            // Font files
            "ttf" | "otf" | "woff" | "woff2" => "Font File".to_string(),
            _ => "Binary Data".to_string(),
        }
    }

    // ===== Search functionality =====

    /// Enter search mode
    pub fn enter_search_mode(&mut self) {
        self.search_mode = true;
        self.search_query.clear();
        self.search_results.clear();
        self.current_match = 0;
    }

    /// Exit search mode (but keep results)
    pub fn exit_search_mode(&mut self) {
        self.search_mode = false;
    }

    /// Clear search completely
    pub fn clear_search(&mut self) {
        self.search_mode = false;
        self.search_query.clear();
        self.search_results.clear();
        self.current_match = 0;
    }

    /// Add character to search query
    pub fn add_search_char(&mut self, c: char) {
        self.search_query.push(c);
    }

    /// Remove last character from search query
    pub fn search_backspace(&mut self) {
        self.search_query.pop();
    }

    /// Perform search and populate search_results
    pub fn perform_search(&mut self) {
        self.search_results.clear();
        self.current_match = 0;

        if self.search_query.is_empty() {
            return;
        }

        let query_lower = self.search_query.to_lowercase();

        // Search through content lines
        for (line_idx, line) in self.content.iter().enumerate() {
            if line.to_lowercase().contains(&query_lower) {
                self.search_results.push(line_idx);
            }
        }

        // Auto-scroll to first match
        if !self.search_results.is_empty() {
            self.scroll_to_match(0);
        }
    }

    /// Go to next search match
    pub fn next_match(&mut self) {
        if self.search_results.is_empty() {
            return;
        }

        self.current_match = (self.current_match + 1) % self.search_results.len();
        self.scroll_to_match(self.current_match);
    }

    /// Go to previous search match
    pub fn prev_match(&mut self) {
        if self.search_results.is_empty() {
            return;
        }

        if self.current_match == 0 {
            self.current_match = self.search_results.len() - 1;
        } else {
            self.current_match -= 1;
        }
        self.scroll_to_match(self.current_match);
    }

    /// Scroll to specific match
    fn scroll_to_match(&mut self, match_idx: usize) {
        if match_idx >= self.search_results.len() {
            return;
        }

        let target_line = self.search_results[match_idx];
        // Center the match on screen (approximately)
        self.scroll = target_line.saturating_sub(5);
    }

    /// Get match info string for display
    pub fn get_match_info(&self) -> String {
        if self.search_results.is_empty() {
            "No matches".to_string()
        } else {
            format!("Match {}/{}", self.current_match + 1, self.search_results.len())
        }
    }

    /// Check if a line has a match
    pub fn line_has_match(&self, line_idx: usize) -> bool {
        self.search_results.contains(&line_idx)
    }

    /// Check if a line is the current match
    pub fn is_current_match(&self, line_idx: usize) -> bool {
        if self.search_results.is_empty() {
            return false;
        }
        self.search_results.get(self.current_match) == Some(&line_idx)
    }

    /// Format file information string
    pub fn format_file_info(&self) -> String {
        if self.current_path.as_os_str().is_empty() {
            return String::new();
        }

        let file_name = self.current_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown");

        // Format file size
        let size_str = format_file_size(self.current_size);

        // Get line count
        let lines_count = self.content.len();
        let lines_info = if lines_count >= 1000 {
            format!("{}+ lines", lines_count)
        } else {
            format!("{} lines", lines_count)
        };

        // Format permissions
        let permissions_str = format_permissions(self.current_permissions);

        // Add search info if there are search results
        let search_info = if !self.search_results.is_empty() {
            format!(" | Search: {} matches for '{}'", self.search_results.len(), self.search_query)
        } else if !self.search_query.is_empty() {
            format!(" | Search: no matches for '{}'", self.search_query)
        } else {
            String::new()
        };

        // Add visual mode indicator
        let visual_info = if self.visual_mode {
            let (start, end) = self.get_selection_range();
            format!(" | VISUAL: {} lines", end.saturating_sub(start) + 1)
        } else {
            String::new()
        };

        format!(" {} | {} | {} | {}{}{}", file_name, size_str, lines_info, permissions_str, search_info, visual_info)
    }

    // ===== Visual selection functionality =====

    /// Enter visual selection mode
    pub fn enter_visual_mode(&mut self) {
        self.visual_mode = true;
        // Start selection at current scroll position (top visible line)
        self.visual_start = Some(self.scroll);
        self.visual_cursor = self.scroll;
    }

    /// Exit visual selection mode
    pub fn exit_visual_mode(&mut self) {
        self.visual_mode = false;
        self.visual_start = None;
    }

    /// Move cursor down in visual mode
    pub fn visual_move_down(&mut self) {
        if self.visual_cursor < self.content.len().saturating_sub(1) {
            self.visual_cursor += 1;
        }
    }

    /// Move cursor up in visual mode
    pub fn visual_move_up(&mut self) {
        self.visual_cursor = self.visual_cursor.saturating_sub(1);
    }

    /// Get selection range (start, end) inclusive, always in ascending order
    pub fn get_selection_range(&self) -> (usize, usize) {
        if let Some(start) = self.visual_start {
            let end = self.visual_cursor;
            if start <= end {
                (start, end)
            } else {
                (end, start)
            }
        } else {
            (0, 0)
        }
    }

    /// Check if a line is within the visual selection
    pub fn is_line_selected(&self, line_idx: usize) -> bool {
        if !self.visual_mode {
            return false;
        }
        let (start, end) = self.get_selection_range();
        line_idx >= start && line_idx <= end
    }

    /// Get selected text as a string
    pub fn get_selected_text(&self) -> String {
        if !self.visual_mode {
            return String::new();
        }

        let (start, end) = self.get_selection_range();
        self.content
            .iter()
            .enumerate()
            .filter(|(idx, _)| *idx >= start && *idx <= end)
            .map(|(_, line)| line.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Copy selected text to clipboard and exit visual mode
    pub fn copy_selection(&mut self) -> anyhow::Result<()> {
        if !self.visual_mode {
            return Ok(());
        }

        let text = self.get_selected_text();
        if !text.is_empty() {
            let mut clipboard = arboard::Clipboard::new()?;
            clipboard.set_text(text)?;
        }

        self.exit_visual_mode();
        Ok(())
    }

    /// Update scroll to keep cursor visible in visual mode
    pub fn ensure_visual_cursor_visible(&mut self, visible_height: usize) {
        if !self.visual_mode {
            return;
        }

        let max_scroll = self.content.len().saturating_sub(visible_height);

        // If cursor is above visible area, scroll up
        if self.visual_cursor < self.scroll {
            self.scroll = self.visual_cursor;
        }
        // If cursor is below visible area, scroll down
        else if self.visual_cursor >= self.scroll + visible_height {
            self.scroll = self.visual_cursor.saturating_sub(visible_height - 1).min(max_scroll);
        }
    }
}

/// Format file size in human-readable format
pub fn format_file_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

/// Format Unix permissions as string
pub fn format_permissions(mode: u32) -> String {
    // Extract permission bits (last 9 bits)
    let perms = mode & 0o777;

    // Determine file type
    let file_type = if mode & 0o170000 == 0o040000 {
        'd' // directory
    } else if mode & 0o170000 == 0o120000 {
        'l' // symbolic link
    } else {
        '-' // regular file
    };

    // Format permissions for owner, group, and others
    let user = format_permission_triplet((perms >> 6) & 0o7);
    let group = format_permission_triplet((perms >> 3) & 0o7);
    let other = format_permission_triplet(perms & 0o7);

    format!("{}{}{}{} ({:04o})", file_type, user, group, other, perms)
}

fn format_permission_triplet(triplet: u32) -> String {
    let r = if triplet & 0o4 != 0 { 'r' } else { '-' };
    let w = if triplet & 0o2 != 0 { 'w' } else { '-' };
    let x = if triplet & 0o1 != 0 { 'x' } else { '-' };
    format!("{}{}{}", r, w, x)
}
