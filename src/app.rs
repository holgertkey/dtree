use std::path::PathBuf;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::Frame;
use anyhow::Result;

use crate::navigation::Navigation;
use crate::file_viewer::FileViewer;
use crate::search::Search;
use crate::ui::UI;
use crate::event_handler::EventHandler;
use crate::config::Config;
use crate::bookmarks::Bookmarks;
use crate::dir_size::DirSizeCache;

/// Main application state
pub struct App {
    nav: Navigation,
    file_viewer: FileViewer,
    search: Search,
    ui: UI,
    event_handler: EventHandler,
    config: Config,
    pub bookmarks: Bookmarks,
    show_files: bool,
    show_files_before_help: bool,
    show_help: bool,
    fullscreen_viewer: bool,
    show_sizes: bool,
    dir_size_cache: DirSizeCache,
}

impl App {
    pub fn new(start_path: PathBuf) -> Result<Self> {
        // Load configuration from global config file
        let config = Config::load();

        let nav = Navigation::new(start_path, false, config.behavior.show_hidden, config.behavior.follow_symlinks)?;
        let mut file_viewer = FileViewer::new();
        let search = Search::new();
        let mut ui = UI::new();
        let event_handler = EventHandler::new();
        let bookmarks = Bookmarks::new()?;

        // Apply config to UI and file viewer
        ui.split_position = config.appearance.split_position;
        file_viewer.show_line_numbers = config.appearance.show_line_numbers;

        Ok(App {
            nav,
            file_viewer,
            search,
            ui,
            event_handler,
            config,
            bookmarks,
            show_files: false,
            show_files_before_help: false,
            show_help: false,
            fullscreen_viewer: false,
            show_sizes: false,
            dir_size_cache: DirSizeCache::new(),
        })
    }

    /// Get reference to the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Result<Option<PathBuf>> {
        self.event_handler.handle_key(
            key,
            &mut self.nav,
            &mut self.file_viewer,
            &mut self.search,
            &mut self.bookmarks,
            &mut self.show_files,
            &mut self.show_files_before_help,
            &mut self.show_help,
            &mut self.fullscreen_viewer,
            &mut self.show_sizes,
            &mut self.dir_size_cache,
            &self.ui,
            &self.config,
        )
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent) -> Result<()> {
        self.event_handler.handle_mouse(
            mouse,
            &mut self.nav,
            &mut self.file_viewer,
            &mut self.search,
            &mut self.bookmarks,
            &mut self.ui,
            &mut self.show_files,
            &mut self.show_help,
            self.fullscreen_viewer,
            &self.config,
        )
    }

    pub fn render(&mut self, frame: &mut Frame) {
        self.ui.render(
            frame,
            &self.nav,
            &self.file_viewer,
            &self.search,
            &self.bookmarks,
            &self.config,
            self.show_files,
            self.show_help,
            self.fullscreen_viewer,
            self.show_sizes,
            &self.dir_size_cache,
        );
    }

    /// Poll search results from background thread
    /// Returns true if there were updates and UI needs to be redrawn
    pub fn poll_search(&mut self) -> bool {
        self.search.poll_results()
    }

    /// Poll directory size calculation results from background thread
    /// Returns true if there were updates and UI needs to be redrawn
    pub fn poll_sizes(&mut self) -> bool {
        self.dir_size_cache.poll_results()
    }

    /// Set fullscreen viewer mode and load the specified file
    pub fn set_fullscreen_viewer(&mut self, file_path: &std::path::Path) -> Result<()> {
        self.fullscreen_viewer = true;
        self.show_files = true;

        // Reload tree with files enabled (so we can navigate between files with Ctrl+j/k)
        self.nav.reload_tree(true)?;

        // Find and select the current file in the flat list
        if let Some(index) = self.nav.flat_list.iter().position(|node| {
            node.borrow().path == file_path
        }) {
            self.nav.selected = index;
        }

        // Load file with very large width for fullscreen (terminal width unknown at this point)
        let max_lines = self.config.behavior.max_file_lines;
        let enable_highlighting = self.config.appearance.enable_syntax_highlighting;
        let theme = &self.config.appearance.syntax_theme.clone();
        self.file_viewer.load_file_with_width(file_path, None, max_lines, enable_highlighting, theme)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_help_toggle_restores_show_files_state() {
        // Test case 1: show_files was false before opening help
        let temp_dir = std::env::temp_dir().join("dtree_test_1");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let mut app = App::new(temp_dir.clone()).unwrap();

        // Initially show_files should be false
        assert!(!app.show_files);
        assert!(!app.show_files_before_help);
        assert!(!app.show_help);

        // Open help (press 'i')
        let key_i = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);
        let _ = app.handle_key(key_i);

        // After opening help, show_files should be true, but previous state saved as false
        assert!(app.show_files);
        assert!(!app.show_files_before_help);
        assert!(app.show_help);

        // Close help (press 'i' again)
        let _ = app.handle_key(key_i);

        // After closing help, show_files should be restored to false
        assert!(!app.show_files);
        assert!(!app.show_files_before_help);
        assert!(!app.show_help);

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_help_toggle_preserves_show_files_when_already_true() {
        // Test case 2: show_files was true before opening help
        let temp_dir = std::env::temp_dir().join("dtree_test_2");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let mut app = App::new(temp_dir.clone()).unwrap();

        // Enable show_files first (press 's')
        let key_s = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
        let _ = app.handle_key(key_s);

        // Now show_files should be true
        assert!(app.show_files);
        assert!(!app.show_help);

        // Open help (press 'i')
        let key_i = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);
        let _ = app.handle_key(key_i);

        // After opening help, show_files still true, previous state saved as true
        assert!(app.show_files);
        assert!(app.show_files_before_help);
        assert!(app.show_help);

        // Close help (press 'i' again)
        let _ = app.handle_key(key_i);

        // After closing help, show_files should still be true
        assert!(app.show_files);
        assert!(app.show_files_before_help);
        assert!(!app.show_help);

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_multiple_help_toggles() {
        // Test case 3: Multiple open/close cycles
        let temp_dir = std::env::temp_dir().join("dtree_test_3");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let mut app = App::new(temp_dir.clone()).unwrap();

        let key_i = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);

        // Initially false
        assert!(!app.show_files);

        // First cycle: open and close
        let _ = app.handle_key(key_i); // open
        assert!(app.show_help);
        let _ = app.handle_key(key_i); // close
        assert!(!app.show_help);
        assert!(!app.show_files); // should be restored

        // Second cycle: open and close
        let _ = app.handle_key(key_i); // open
        assert!(app.show_help);
        let _ = app.handle_key(key_i); // close
        assert!(!app.show_help);
        assert!(!app.show_files); // should be restored again

        std::fs::remove_dir_all(&temp_dir).ok();
    }
}
