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
static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(|| SyntaxSet::load_defaults_newlines());

/// Lazy-loaded theme set (loaded once on first use)
static THEME_SET: Lazy<ThemeSet> = Lazy::new(|| ThemeSet::load_defaults());

/// File viewer state and logic for displaying file contents
pub struct FileViewer {
    pub content: Vec<String>,
    pub highlighted_content: Vec<Line<'static>>,
    pub scroll: usize,
    pub current_path: PathBuf,
    pub current_size: u64,
    pub current_permissions: u32,
    pub show_line_numbers: bool,
    pub syntax_name: Option<String>,
    pub is_binary: bool,
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
            syntax_name: None,
            is_binary: false,
        }
    }

    /// Toggle line numbers display
    pub fn toggle_line_numbers(&mut self) {
        self.show_line_numbers = !self.show_line_numbers;
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

        // Try to open file
        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                self.content.push(format!("[Error: {}]", e));
                return Ok(());
            }
        };

        let reader = BufReader::new(file);
        let mut line_count = 0;

        for line in reader.lines() {
            if line_count >= max_lines {
                self.content.push(format!("\n[... truncated at {} lines ...]", max_lines));
                break;
            }

            match line {
                Ok(content) => {
                    // Replace tabs with spaces (4 spaces per tab)
                    let content_no_tabs = content.replace('\t', "    ");
                    // Truncate line to prevent Unicode artifacts
                    let truncated = Self::truncate_line(&content_no_tabs, max_width);
                    self.content.push(truncated);
                    line_count += 1;
                }
                Err(e) => {
                    // Possibly binary file or encoding error
                    self.content.clear();
                    self.content.push(format!("[Binary file or encoding error: {}]", e));
                    break;
                }
            }
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

    /// Truncate a line to max_width using Unicode-aware truncation
    fn truncate_line(line: &str, max_width: usize) -> String {
        // Use visual width, not byte length
        let line_width = line.width();

        if line_width <= max_width {
            return line.to_string();
        }

        // Use unicode-aware truncation
        let (truncated, _) = line.unicode_truncate(max_width.saturating_sub(3));
        if truncated.len() < line.len() {
            format!("{}...", truncated)
        } else {
            truncated.to_string()
        }
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

        format!(" {} | {} | {} | {}", file_name, size_str, lines_info, permissions_str)
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
