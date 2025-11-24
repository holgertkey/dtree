// Export modules for testing
pub mod bookmarks;
pub mod config;
pub mod dir_size;
pub mod event_handler;
pub mod file_icons;
pub mod file_viewer;
pub mod navigation;
pub mod search;
pub mod theme;
pub mod tree_node;
pub mod ui;

// Re-export app module (not public but tests need access)
pub mod app;
