use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::unix::fs::PermissionsExt;
use anyhow::Result;
use unicode_truncate::UnicodeTruncateStr;
use unicode_width::UnicodeWidthStr;

/// File viewer state and logic for displaying file contents
pub struct FileViewer {
    pub content: Vec<String>,
    pub scroll: usize,
    pub current_path: PathBuf,
    pub current_size: u64,
    pub current_permissions: u32,
}

impl FileViewer {
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
            scroll: 0,
            current_path: PathBuf::new(),
            current_size: 0,
            current_permissions: 0,
        }
    }

    /// Load file content with specified max width and max lines
    pub fn load_file_with_width(&mut self, path: &Path, max_width: Option<usize>, max_lines: usize) -> Result<()> {
        const DEFAULT_MAX_WIDTH: usize = 10000; // Very large default to avoid truncation

        let max_width = max_width.unwrap_or(DEFAULT_MAX_WIDTH);

        self.content.clear();
        self.scroll = 0;
        self.current_path = path.to_path_buf();
        self.current_size = 0;
        self.current_permissions = 0;

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
                    // Truncate line to prevent Unicode artifacts
                    let truncated = Self::truncate_line(&content, max_width);
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

        Ok(())
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

    /// Reset scroll position
    pub fn reset_scroll(&mut self) {
        self.scroll = 0;
    }

    /// Load custom content (e.g., help text)
    pub fn load_content(&mut self, content: Vec<String>) {
        self.content = content;
        self.scroll = 0;
        self.current_path = PathBuf::new();
        self.current_size = 0;
        self.current_permissions = 0;
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
