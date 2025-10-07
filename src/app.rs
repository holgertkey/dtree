use std::path::PathBuf;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::Frame;
use anyhow::Result;

use crate::navigation::Navigation;
use crate::file_viewer::FileViewer;
use crate::search::Search;
use crate::ui::UI;
use crate::event_handler::EventHandler;

/// Main application state
pub struct App {
    nav: Navigation,
    file_viewer: FileViewer,
    search: Search,
    ui: UI,
    event_handler: EventHandler,
    show_files: bool,
    show_help: bool,
    fullscreen_viewer: bool,
}

impl App {
    pub fn new(start_path: PathBuf) -> Result<Self> {
        let nav = Navigation::new(start_path, false)?;
        let file_viewer = FileViewer::new();
        let search = Search::new();
        let ui = UI::new();
        let event_handler = EventHandler::new();

        Ok(App {
            nav,
            file_viewer,
            search,
            ui,
            event_handler,
            show_files: false,
            show_help: false,
            fullscreen_viewer: false,
        })
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Result<Option<PathBuf>> {
        self.event_handler.handle_key(
            key,
            &mut self.nav,
            &mut self.file_viewer,
            &mut self.search,
            &mut self.show_files,
            &mut self.show_help,
            &mut self.fullscreen_viewer,
            &self.ui,
        )
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent) -> Result<()> {
        self.event_handler.handle_mouse(
            mouse,
            &mut self.nav,
            &mut self.file_viewer,
            &mut self.ui,
            self.show_files,
            &mut self.show_help,
        )
    }

    pub fn render(&mut self, frame: &mut Frame) {
        self.ui.render(
            frame,
            &self.nav,
            &self.file_viewer,
            &self.search,
            self.show_files,
            self.show_help,
            self.fullscreen_viewer,
        );
    }

    /// Set fullscreen viewer mode and load the specified file
    pub fn set_fullscreen_viewer(&mut self, file_path: &std::path::Path) -> Result<()> {
        self.fullscreen_viewer = true;
        self.show_files = true;
        // Load file with very large width for fullscreen (terminal width unknown at this point)
        self.file_viewer.load_file_with_width(file_path, Some(10000))?;
        Ok(())
    }
}
