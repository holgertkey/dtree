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
        );
    }
}
