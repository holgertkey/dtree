use std::path::PathBuf;
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use crossterm::{
    event::{self, Event, EnableMouseCapture, DisableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use anyhow::Result;

use crate::app::App;

pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stderr>>> {
    enable_raw_mode()?;
    std::io::stderr().execute(EnterAlternateScreen)?;
    std::io::stderr().execute(EnableMouseCapture)?;

    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;

    Ok(terminal)
}

pub fn cleanup_terminal() -> Result<()> {
    disable_raw_mode()?;
    std::io::stderr().execute(DisableMouseCapture)?;
    std::io::stderr().execute(LeaveAlternateScreen)?;
    Ok(())
}

pub fn run_app(terminal: &mut Terminal<CrosstermBackend<std::io::Stderr>>, app: &mut App) -> Result<Option<PathBuf>> {
    loop {
        terminal.draw(|f| app.render(f))?;

        match event::read()? {
            Event::Key(key) => {
                match app.handle_key(key)? {
                    Some(path) if !path.as_os_str().is_empty() => {
                        return Ok(Some(path));
                    }
                    None => {
                        return Ok(None);
                    }
                    _ => {}
                }
            }
            Event::Mouse(mouse) => {
                app.handle_mouse(mouse);
            }
            _ => {}
        }
    }
}
