// Export modules for testing
pub mod tree_node;
pub mod file_viewer;
pub mod navigation;
pub mod search;
pub mod config;
pub mod theme;
pub mod bookmarks;
pub mod ui;
pub mod event_handler;
pub mod dir_size;
pub mod file_icons;

// Re-export app module (not public but tests need access)
pub mod app;
