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
    use crossterm::terminal::{Clear, ClearType};

    // Restore terminal state in reverse order of setup

    // 1. CRITICAL: Explicitly disable ALL mouse tracking modes
    //    This is more thorough than just DisableMouseCapture
    let _ = write!(std::io::stderr(), "\x1b[?1000l");  // Disable X10 mouse
    let _ = write!(std::io::stderr(), "\x1b[?1002l");  // Disable cell motion
    let _ = write!(std::io::stderr(), "\x1b[?1003l");  // Disable all motion
    let _ = write!(std::io::stderr(), "\x1b[?1006l");  // Disable SGR mode
    let _ = write!(std::io::stderr(), "\x1b[?1015l");  // Disable urxvt mode
    let _ = std::io::stderr().execute(DisableMouseCapture);
    let _ = std::io::stderr().flush();

    // 2. Give terminal MORE time to process mouse disable commands
    //    Increased to 20ms to handle slow terminals
    std::thread::sleep(std::time::Duration::from_millis(20));

    // 3. First aggressive drain of pending events
    let mut drain_count = 0;
    while event::poll(std::time::Duration::from_millis(0)).unwrap_or(false) && drain_count < 100 {
        let _ = event::read();
        drain_count += 1;
    }

    // 4. Clear alternate screen before leaving it
    let _ = std::io::stderr().execute(Clear(ClearType::All));
    let _ = std::io::stderr().flush();

    // 5. Leave alternate screen
    let _ = std::io::stderr().execute(LeaveAlternateScreen);
    let _ = std::io::stderr().flush();

    // 6. IMPORTANT: Another delay + drain AFTER leaving alternate screen
    //    Sometimes events leak during the screen transition
    std::thread::sleep(std::time::Duration::from_millis(10));

    let mut drain_count2 = 0;
    while event::poll(std::time::Duration::from_millis(0)).unwrap_or(false) && drain_count2 < 50 {
        let _ = event::read();
        drain_count2 += 1;
    }

    // 7. Disable raw mode (this should stop all special terminal modes)
    let _ = disable_raw_mode();

    // 8. Send minimal reset sequences (no screen clearing!)
    //    Reset character attributes (SGR 0)
    let _ = write!(std::io::stderr(), "\x1b[0m");
    //    Show cursor
    let _ = write!(std::io::stderr(), "\x1b[?25h");
    let _ = std::io::stderr().flush();

    // 9. Final delay to ensure terminal processes everything
    std::thread::sleep(std::time::Duration::from_millis(10));

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
                Event::Resize(_width, _height) => {
                    // Terminal was resized - the next draw() will handle it automatically
                    // Just consume the event to prevent it from leaking
                }
                _ => {
                    // Consume all other events (FocusGained, FocusLost, Paste, etc.)
                }
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
