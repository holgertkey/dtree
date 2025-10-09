use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// A single bookmark entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub key: char,
    pub path: PathBuf,
    pub name: Option<String>,
}

/// Manages persistent bookmarks
#[derive(Debug, Default)]
pub struct Bookmarks {
    bookmarks: HashMap<char, Bookmark>,
    file_path: PathBuf,
    pub is_selecting: bool,
    pub pending_key: Option<char>,
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
            pending_key: None,
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

        // Try to parse JSON, if fails - backup the corrupted file and start fresh
        match serde_json::from_str::<Vec<Bookmark>>(&content) {
            Ok(bookmarks_vec) => {
                self.bookmarks.clear();
                for bookmark in bookmarks_vec {
                    self.bookmarks.insert(bookmark.key, bookmark);
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
    pub fn add(&mut self, key: char, path: PathBuf, name: Option<String>) -> Result<()> {
        let bookmark = Bookmark {
            key,
            path,
            name,
        };

        self.bookmarks.insert(key, bookmark);
        self.save()?;
        Ok(())
    }

    /// Get a bookmark by key
    pub fn get(&self, key: char) -> Option<&Bookmark> {
        self.bookmarks.get(&key)
    }

    /// Remove a bookmark
    pub fn remove(&mut self, key: char) -> Result<()> {
        self.bookmarks.remove(&key);
        self.save()?;
        Ok(())
    }

    /// Get all bookmarks as a sorted vector
    pub fn list(&self) -> Vec<&Bookmark> {
        let mut bookmarks: Vec<&Bookmark> = self.bookmarks.values().collect();
        bookmarks.sort_by_key(|b| b.key);
        bookmarks
    }

    /// Check if a key is already used
    pub fn has_key(&self, key: char) -> bool {
        self.bookmarks.contains_key(&key)
    }

    /// Enter bookmark selection mode
    pub fn enter_selection_mode(&mut self) {
        self.is_selecting = true;
        self.pending_key = None;
    }

    /// Exit bookmark selection mode
    pub fn exit_selection_mode(&mut self) {
        self.is_selecting = false;
        self.pending_key = None;
    }

    /// Enter bookmark creation mode (after pressing 'm')
    pub fn enter_creation_mode(&mut self) {
        self.pending_key = Some('m');
    }

    /// Exit bookmark creation mode
    pub fn exit_creation_mode(&mut self) {
        self.pending_key = None;
    }

    /// Check if in creation mode
    pub fn is_in_creation_mode(&self) -> bool {
        self.pending_key.is_some()
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
            pending_key: None,
        }
    }

    #[test]
    fn test_save_and_load_bookmarks() {
        let temp_dir = TempDir::new().unwrap();
        let mut bookmarks = create_test_bookmarks(&temp_dir);

        // Add some bookmarks
        bookmarks.add('a', PathBuf::from("/tmp/test1"), Some("Test 1".to_string())).unwrap();
        bookmarks.add('b', PathBuf::from("/tmp/test2"), Some("Test 2".to_string())).unwrap();

        // Save
        bookmarks.save().unwrap();

        // Create new instance and load
        let mut bookmarks2 = create_test_bookmarks(&temp_dir);
        bookmarks2.load().unwrap();

        // Verify
        assert_eq!(bookmarks2.list().len(), 2);
        assert_eq!(bookmarks2.get('a').unwrap().path, PathBuf::from("/tmp/test1"));
        assert_eq!(bookmarks2.get('b').unwrap().path, PathBuf::from("/tmp/test2"));
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
            pending_key: None,
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
            pending_key: None,
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
        bookmarks.add('x', PathBuf::from("/tmp/testx"), Some("TestX".to_string())).unwrap();
        assert_eq!(bookmarks.list().len(), 1);
        assert!(bookmarks.has_key('x'));

        // Remove bookmark
        bookmarks.remove('x').unwrap();
        assert_eq!(bookmarks.list().len(), 0);
        assert!(!bookmarks.has_key('x'));
    }

    #[test]
    fn test_list_sorted_by_key() {
        let temp_dir = TempDir::new().unwrap();
        let mut bookmarks = create_test_bookmarks(&temp_dir);

        // Add bookmarks in random order
        bookmarks.add('z', PathBuf::from("/tmp/z"), None).unwrap();
        bookmarks.add('a', PathBuf::from("/tmp/a"), None).unwrap();
        bookmarks.add('m', PathBuf::from("/tmp/m"), None).unwrap();

        let list = bookmarks.list();
        assert_eq!(list.len(), 3);

        // Should be sorted by key
        assert_eq!(list[0].key, 'a');
        assert_eq!(list[1].key, 'm');
        assert_eq!(list[2].key, 'z');
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

        assert!(!bookmarks.is_in_creation_mode());

        bookmarks.enter_creation_mode();
        assert!(bookmarks.is_in_creation_mode());

        bookmarks.exit_creation_mode();
        assert!(!bookmarks.is_in_creation_mode());
    }

    #[test]
    fn test_overwrite_existing_bookmark() {
        let temp_dir = TempDir::new().unwrap();
        let mut bookmarks = create_test_bookmarks(&temp_dir);

        // Add bookmark
        bookmarks.add('t', PathBuf::from("/tmp/first"), Some("First".to_string())).unwrap();
        assert_eq!(bookmarks.get('t').unwrap().path, PathBuf::from("/tmp/first"));

        // Overwrite with same key
        bookmarks.add('t', PathBuf::from("/tmp/second"), Some("Second".to_string())).unwrap();

        // Should have updated path
        assert_eq!(bookmarks.get('t').unwrap().path, PathBuf::from("/tmp/second"));
        assert_eq!(bookmarks.get('t').unwrap().name, Some("Second".to_string()));

        // Should still have only one bookmark
        assert_eq!(bookmarks.list().len(), 1);
    }
}
