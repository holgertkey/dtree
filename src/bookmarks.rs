use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// A single bookmark entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub key: String,
    pub path: PathBuf,
    pub name: Option<String>,
}

impl Bookmark {
    /// Validate bookmark name according to filesystem naming rules
    /// Allows alphanumeric, hyphens, underscores, dots (max 255 chars)
    /// Forbids path separators and null bytes
    pub fn validate_name(name: &str) -> Result<()> {
        if name.is_empty() {
            anyhow::bail!("Bookmark name cannot be empty");
        }

        if name.len() > 255 {
            anyhow::bail!("Bookmark name too long (max 255 characters)");
        }

        // Check for forbidden characters (path separators, null byte, control chars)
        if name.contains('/') || name.contains('\\') || name.contains('\0') {
            anyhow::bail!("Bookmark name cannot contain path separators (/, \\) or null bytes");
        }

        // Forbid control characters
        if name.chars().any(|c| c.is_control()) {
            anyhow::bail!("Bookmark name cannot contain control characters");
        }

        // Forbid reserved names on Windows (optional, but safer for cross-platform)
        let reserved = ["CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4",
                        "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2",
                        "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"];
        if reserved.contains(&name.to_uppercase().as_str()) {
            anyhow::bail!("Bookmark name '{}' is reserved", name);
        }

        Ok(())
    }
}

/// Manages persistent bookmarks
#[derive(Debug, Default)]
pub struct Bookmarks {
    bookmarks: HashMap<String, Bookmark>,
    file_path: PathBuf,
    pub is_selecting: bool,
    pub is_creating: bool,
    pub input_buffer: String,
    pub selected_index: usize,       // Current selection in list
    pub filter_mode: bool,            // True = filter/search mode, False = navigation mode
    filtered_keys: Vec<String>,       // Cached filtered bookmark keys
    pub scroll_offset: usize,         // Scroll offset for bookmark list in creation mode
}

impl Bookmarks {
    /// Create a new Bookmarks instance and load from file
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .context("Could not find config directory")?
            .join("dtree");

        // Ensure config directory exists
        fs::create_dir_all(&config_dir)?;

        let file_path = config_dir.join("bookmarks.json");

        let mut bookmarks = Self {
            bookmarks: HashMap::new(),
            file_path,
            is_selecting: false,
            is_creating: false,
            input_buffer: String::new(),
            selected_index: 0,
            filter_mode: false,
            filtered_keys: Vec::new(),
            scroll_offset: 0,
        };

        // Try to load, but don't fail if JSON is corrupted
        if let Err(e) = bookmarks.load() {
            eprintln!("\n┌─────────────────────────────────────────────────────────────┐");
            eprintln!("│ WARNING: Bookmarks file is corrupted                       │");
            eprintln!("└─────────────────────────────────────────────────────────────┘\n");
            eprintln!("{}\n", e);
            eprintln!("Press Enter to continue...");

            // Wait for user to press Enter
            let mut input = String::new();
            let _ = std::io::stdin().read_line(&mut input);
        }

        Ok(bookmarks)
    }

    /// Load bookmarks from JSON file
    fn load(&mut self) -> Result<()> {
        if !self.file_path.exists() {
            // Create empty file if it doesn't exist
            self.save()?;
            return Ok(());
        }

        let content = match fs::read_to_string(&self.file_path) {
            Ok(c) => c,
            Err(_) => {
                // If cannot read file, create new empty one
                self.save()?;
                return Ok(());
            }
        };

        if content.trim().is_empty() {
            return Ok(());
        }

        // Parse JSON
        match serde_json::from_str::<Vec<Bookmark>>(&content) {
            Ok(bookmarks_vec) => {
                self.bookmarks.clear();
                for bookmark in bookmarks_vec {
                    self.bookmarks.insert(bookmark.key.clone(), bookmark);
                }
                Ok(())
            }
            Err(e) => {
                // Backup corrupted file
                let backup_path = self.file_path.with_extension("json.backup");
                let _ = fs::copy(&self.file_path, &backup_path);

                // Create new empty bookmarks file
                self.bookmarks.clear();
                self.save()?;

                // Return error with helpful message
                Err(anyhow::anyhow!(
                    "Failed to parse bookmarks JSON: {}.\n\
                    The corrupted file has been backed up to: {}\n\
                    A new empty bookmarks file has been created.",
                    e,
                    backup_path.display()
                ))
            }
        }
    }

    /// Save bookmarks to JSON file
    fn save(&self) -> Result<()> {
        let bookmarks_vec: Vec<&Bookmark> = self.bookmarks.values().collect();
        let json = serde_json::to_string_pretty(&bookmarks_vec)
            .context("Failed to serialize bookmarks")?;

        fs::write(&self.file_path, json)
            .context("Failed to write bookmarks file")?;

        Ok(())
    }

    /// Add or update a bookmark
    pub fn add(&mut self, key: String, path: PathBuf, name: Option<String>) -> Result<()> {
        // Validate bookmark name
        Bookmark::validate_name(&key)?;

        let bookmark = Bookmark {
            key: key.clone(),
            path,
            name,
        };

        self.bookmarks.insert(key, bookmark);
        self.save()?;
        Ok(())
    }

    /// Get a bookmark by key
    pub fn get(&self, key: &str) -> Option<&Bookmark> {
        self.bookmarks.get(key)
    }

    /// Remove a bookmark
    pub fn remove(&mut self, key: &str) -> Result<()> {
        if self.bookmarks.remove(key).is_none() {
            anyhow::bail!("Bookmark '{}' not found", key);
        }
        self.save()?;
        Ok(())
    }

    /// Get all bookmarks as a sorted vector
    pub fn list(&self) -> Vec<&Bookmark> {
        let mut bookmarks: Vec<&Bookmark> = self.bookmarks.values().collect();
        bookmarks.sort_by_key(|b| b.key.clone());
        bookmarks
    }

    /// Check if a key is already used
    pub fn has_key(&self, key: &str) -> bool {
        self.bookmarks.contains_key(key)
    }

    /// Enter bookmark selection mode
    pub fn enter_selection_mode(&mut self) {
        self.is_selecting = true;
        self.is_creating = false;
        self.input_buffer.clear();
        self.selected_index = 0;
        self.filter_mode = false;
        self.update_filtered_list();
    }

    /// Exit bookmark selection mode
    pub fn exit_selection_mode(&mut self) {
        self.is_selecting = false;
        self.input_buffer.clear();
        self.selected_index = 0;
        self.filter_mode = false;
        self.filtered_keys.clear();
    }

    /// Enter bookmark creation mode (after pressing 'm')
    pub fn enter_creation_mode(&mut self) {
        self.is_creating = true;
        self.is_selecting = false;
        self.input_buffer.clear();
        self.selected_index = 0;
        self.filter_mode = false;
        self.scroll_offset = 0;
    }

    /// Exit bookmark creation mode
    pub fn exit_creation_mode(&mut self) {
        self.is_creating = false;
        self.input_buffer.clear();
        self.scroll_offset = 0;
    }

    /// Scroll bookmark list up in creation mode
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Scroll bookmark list down in creation mode
    pub fn scroll_down(&mut self, max_visible: usize) {
        let total_bookmarks = self.list().len();
        let max_offset = total_bookmarks.saturating_sub(max_visible);
        if self.scroll_offset < max_offset {
            self.scroll_offset += 1;
        }
    }

    /// Add character to input buffer
    pub fn add_char(&mut self, c: char) {
        self.input_buffer.push(c);
        // Update filtered list if in filter mode
        if self.filter_mode {
            self.update_filtered_list();
            self.selected_index = 0; // Reset selection to top
        }
    }

    /// Remove last character from input buffer
    pub fn backspace(&mut self) {
        self.input_buffer.pop();
        // Update filtered list if in filter mode
        if self.filter_mode {
            self.update_filtered_list();
            self.selected_index = 0; // Reset selection to top
        }
    }

    /// Get current input buffer
    pub fn get_input(&self) -> &str {
        &self.input_buffer
    }

    /// Toggle between navigation mode and filter mode
    pub fn toggle_filter_mode(&mut self) {
        self.filter_mode = !self.filter_mode;
        if self.filter_mode {
            // Entering filter mode - clear input
            self.input_buffer.clear();
        } else {
            // Exiting filter mode - restore full list
            self.input_buffer.clear();
            self.update_filtered_list();
        }
        self.selected_index = 0;
    }

    /// Update filtered list based on input buffer
    fn update_filtered_list(&mut self) {
        let query = self.input_buffer.to_lowercase();

        if query.is_empty() {
            // No filter - show all bookmarks
            self.filtered_keys = self.list().iter().map(|b| b.key.clone()).collect();
        } else {
            // Filter bookmarks by key or name
            self.filtered_keys = self.list()
                .iter()
                .filter(|b| {
                    let key_match = b.key.to_lowercase().contains(&query);
                    let name_match = b.name.as_ref()
                        .map(|n| n.to_lowercase().contains(&query))
                        .unwrap_or(false);
                    key_match || name_match
                })
                .map(|b| b.key.clone())
                .collect();
        }
    }

    /// Get filtered bookmarks for display
    pub fn get_filtered_bookmarks(&self) -> Vec<&Bookmark> {
        if self.filtered_keys.is_empty() {
            Vec::new()
        } else {
            self.filtered_keys
                .iter()
                .filter_map(|key| self.bookmarks.get(key))
                .collect()
        }
    }

    /// Move selection up in bookmark list
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down in bookmark list
    pub fn move_down(&mut self) {
        let list_len = self.get_filtered_bookmarks().len();
        if list_len > 0 && self.selected_index < list_len - 1 {
            self.selected_index += 1;
        }
    }

    /// Get currently selected bookmark
    pub fn get_selected_bookmark(&self) -> Option<&Bookmark> {
        let filtered = self.get_filtered_bookmarks();
        filtered.get(self.selected_index).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    /// Helper function to create a Bookmarks instance with a temporary file
    fn create_test_bookmarks(temp_dir: &TempDir) -> Bookmarks {
        let file_path = temp_dir.path().join("bookmarks.json");
        Bookmarks {
            bookmarks: HashMap::new(),
            file_path,
            is_selecting: false,
            is_creating: false,
            input_buffer: String::new(),
            selected_index: 0,
            filter_mode: false,
            filtered_keys: Vec::new(),
            scroll_offset: 0,
        }
    }

    #[test]
    fn test_save_and_load_bookmarks() {
        let temp_dir = TempDir::new().unwrap();
        let mut bookmarks = create_test_bookmarks(&temp_dir);

        // Add some bookmarks
        bookmarks.add("a".to_string(), PathBuf::from("/tmp/test1"), Some("Test 1".to_string())).unwrap();
        bookmarks.add("b".to_string(), PathBuf::from("/tmp/test2"), Some("Test 2".to_string())).unwrap();

        // Save
        bookmarks.save().unwrap();

        // Create new instance and load
        let mut bookmarks2 = create_test_bookmarks(&temp_dir);
        bookmarks2.load().unwrap();

        // Verify
        assert_eq!(bookmarks2.list().len(), 2);
        assert_eq!(bookmarks2.get("a").unwrap().path, PathBuf::from("/tmp/test1"));
        assert_eq!(bookmarks2.get("b").unwrap().path, PathBuf::from("/tmp/test2"));
    }

    #[test]
    fn test_corrupted_json_creates_backup() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("bookmarks.json");
        let backup_path = temp_dir.path().join("bookmarks.json.backup");

        // Write invalid JSON (trailing comma)
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"[\n  {\"key\": \"a\", \"path\": \"/tmp/test\", \"name\": \"test\"},\n]").unwrap();
        drop(file);

        // Try to load
        let mut bookmarks = Bookmarks {
            bookmarks: HashMap::new(),
            file_path: file_path.clone(),
            is_selecting: false,
            is_creating: false,
            input_buffer: String::new(),
            selected_index: 0,
            filter_mode: false,
            filtered_keys: Vec::new(),
            scroll_offset: 0,
        };

        let result = bookmarks.load();

        // Should return error
        assert!(result.is_err());

        // Backup should exist
        assert!(backup_path.exists());

        // New file should be valid (empty array)
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "[]");

        // Bookmarks should be empty
        assert_eq!(bookmarks.list().len(), 0);
    }

    #[test]
    fn test_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("bookmarks.json");

        // Create empty file
        std::fs::File::create(&file_path).unwrap();

        let mut bookmarks = Bookmarks {
            bookmarks: HashMap::new(),
            file_path,
            is_selecting: false,
            is_creating: false,
            input_buffer: String::new(),
            selected_index: 0,
            filter_mode: false,
            filtered_keys: Vec::new(),
            scroll_offset: 0,
        };

        // Should load without error
        bookmarks.load().unwrap();
        assert_eq!(bookmarks.list().len(), 0);
    }

    #[test]
    fn test_add_and_remove_bookmarks() {
        let temp_dir = TempDir::new().unwrap();
        let mut bookmarks = create_test_bookmarks(&temp_dir);

        // Add bookmark
        bookmarks.add("x".to_string(), PathBuf::from("/tmp/testx"), Some("TestX".to_string())).unwrap();
        assert_eq!(bookmarks.list().len(), 1);
        assert!(bookmarks.has_key("x"));

        // Remove bookmark
        bookmarks.remove("x").unwrap();
        assert_eq!(bookmarks.list().len(), 0);
        assert!(!bookmarks.has_key("x"));
    }

    #[test]
    fn test_list_sorted_by_key() {
        let temp_dir = TempDir::new().unwrap();
        let mut bookmarks = create_test_bookmarks(&temp_dir);

        // Add bookmarks in random order
        bookmarks.add("z".to_string(), PathBuf::from("/tmp/z"), None).unwrap();
        bookmarks.add("a".to_string(), PathBuf::from("/tmp/a"), None).unwrap();
        bookmarks.add("m".to_string(), PathBuf::from("/tmp/m"), None).unwrap();

        let list = bookmarks.list();
        assert_eq!(list.len(), 3);

        // Should be sorted by key
        assert_eq!(list[0].key, "a");
        assert_eq!(list[1].key, "m");
        assert_eq!(list[2].key, "z");
    }

    #[test]
    fn test_selection_mode() {
        let temp_dir = TempDir::new().unwrap();
        let mut bookmarks = create_test_bookmarks(&temp_dir);

        assert!(!bookmarks.is_selecting);

        bookmarks.enter_selection_mode();
        assert!(bookmarks.is_selecting);

        bookmarks.exit_selection_mode();
        assert!(!bookmarks.is_selecting);
    }

    #[test]
    fn test_creation_mode() {
        let temp_dir = TempDir::new().unwrap();
        let mut bookmarks = create_test_bookmarks(&temp_dir);

        assert!(!bookmarks.is_creating);

        bookmarks.enter_creation_mode();
        assert!(bookmarks.is_creating);

        bookmarks.exit_creation_mode();
        assert!(!bookmarks.is_creating);
    }

    #[test]
    fn test_overwrite_existing_bookmark() {
        let temp_dir = TempDir::new().unwrap();
        let mut bookmarks = create_test_bookmarks(&temp_dir);

        // Add bookmark
        bookmarks.add("t".to_string(), PathBuf::from("/tmp/first"), Some("First".to_string())).unwrap();
        assert_eq!(bookmarks.get("t").unwrap().path, PathBuf::from("/tmp/first"));

        // Overwrite with same key
        bookmarks.add("t".to_string(), PathBuf::from("/tmp/second"), Some("Second".to_string())).unwrap();

        // Should have updated path
        assert_eq!(bookmarks.get("t").unwrap().path, PathBuf::from("/tmp/second"));
        assert_eq!(bookmarks.get("t").unwrap().name, Some("Second".to_string()));

        // Should still have only one bookmark
        assert_eq!(bookmarks.list().len(), 1);
    }

    #[test]
    fn test_multi_character_bookmark_names() {
        let temp_dir = TempDir::new().unwrap();
        let mut bookmarks = create_test_bookmarks(&temp_dir);

        // Add multi-character bookmarks
        bookmarks.add("work".to_string(), PathBuf::from("/tmp/work"), Some("Work".to_string())).unwrap();
        bookmarks.add("project-123".to_string(), PathBuf::from("/tmp/proj"), Some("Project".to_string())).unwrap();
        bookmarks.add("my_home".to_string(), PathBuf::from("/home/user"), Some("Home".to_string())).unwrap();

        assert_eq!(bookmarks.list().len(), 3);
        assert_eq!(bookmarks.get("work").unwrap().path, PathBuf::from("/tmp/work"));
        assert_eq!(bookmarks.get("project-123").unwrap().path, PathBuf::from("/tmp/proj"));
        assert_eq!(bookmarks.get("my_home").unwrap().path, PathBuf::from("/home/user"));
    }

    #[test]
    fn test_bookmark_name_validation() {
        let temp_dir = TempDir::new().unwrap();
        let mut bookmarks = create_test_bookmarks(&temp_dir);

        // Empty name should fail
        let result = bookmarks.add("".to_string(), PathBuf::from("/tmp/test"), None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));

        // Name with path separator should fail
        let result = bookmarks.add("work/project".to_string(), PathBuf::from("/tmp/test"), None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("path separators"));

        // Name with null byte should fail
        let result = bookmarks.add("work\0test".to_string(), PathBuf::from("/tmp/test"), None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("path separators"));

        // Reserved Windows name should fail (cross-platform safety)
        let result = bookmarks.add("CON".to_string(), PathBuf::from("/tmp/test"), None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("reserved"));

        // Valid names should succeed
        assert!(bookmarks.add("work".to_string(), PathBuf::from("/tmp/work"), None).is_ok());
        assert!(bookmarks.add("project-123".to_string(), PathBuf::from("/tmp/proj"), None).is_ok());
        assert!(bookmarks.add("my_home.backup".to_string(), PathBuf::from("/tmp/home"), None).is_ok());
    }

    #[test]
    fn test_bookmark_remove_error() {
        let temp_dir = TempDir::new().unwrap();
        let mut bookmarks = create_test_bookmarks(&temp_dir);

        // Removing non-existent bookmark should fail
        let result = bookmarks.remove("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

}
