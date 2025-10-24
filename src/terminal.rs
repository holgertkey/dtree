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

/// Install panic hook to ensure terminal is always cleaned up
pub fn install_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Try to clean up terminal before panicking
        let _ = cleanup_terminal();
        original_hook(panic_info);
    }));
}

pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stderr>>> {
    // Install panic hook before any terminal modifications
    install_panic_hook();

    enable_raw_mode()?;
    std::io::stderr().execute(EnterAlternateScreen)?;
    std::io::stderr().execute(EnableMouseCapture)?;

    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;

    Ok(terminal)
}

pub fn cleanup_terminal() -> Result<()> {
    use std::io::Write;

    // Restore terminal state in reverse order of setup
    // 1. Disable mouse capture
    let _ = std::io::stderr().execute(DisableMouseCapture);

    // 2. Leave alternate screen
    let _ = std::io::stderr().execute(LeaveAlternateScreen);

    // 3. Disable raw mode
    let _ = disable_raw_mode();

    // 4. Final flush to ensure all commands are sent
    let _ = std::io::stderr().flush();

    Ok(())
}

pub fn run_app(terminal: &mut Terminal<CrosstermBackend<std::io::Stderr>>, app: &mut App) -> Result<Option<PathBuf>> {
    // If in fullscreen mode and file loaded with unknown width, reload with correct terminal width
    if app.is_fullscreen_viewer() {
        let terminal_size = terminal.size()?;
        app.reload_fullscreen_file(terminal_size.width)?;
    }

    loop {
        // Check if terminal needs to be cleared (e.g., after exiting fullscreen mode)
        if app.should_clear_terminal() {
            terminal.clear()?;
        }

        terminal.draw(|f| app.render(f))?;

        // Poll for events with 100ms timeout to allow background search to update
        if event::poll(std::time::Duration::from_millis(100))? {
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
                    let _ = app.handle_mouse(mouse);
                }
                _ => {}
            }
        } else {
            // No event - poll search results if search is active
            let search_updated = app.poll_search();
            let sizes_updated = app.poll_sizes();

            if search_updated || sizes_updated {
                // Updates available - redraw will happen on next loop
            }
        }
    }
}
