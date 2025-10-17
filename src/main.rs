mod tree_node;
mod app;
mod terminal;
mod navigation;
mod file_viewer;
mod search;
mod ui;
mod event_handler;
mod config;
mod theme;
mod bookmarks;
mod dir_size;
mod file_icons;

use anyhow::Result;
use app::App;
use terminal::{setup_terminal, cleanup_terminal, run_app};
use clap::Parser;
use std::path::PathBuf;
use std::process::Command;
use config::Config;
use bookmarks::Bookmarks;

#[derive(Parser)]
#[command(name = "dtree")]
#[command(about = "Interactive directory tree navigator")]
#[command(disable_help_flag = true)]
#[command(disable_version_flag = true)]
struct Args {
    /// Print help information
    #[arg(short = 'h', long = "help")]
    help: bool,

    /// View file directly in fullscreen mode
    #[arg(short = 'v', long = "view", conflicts_with = "version")]
    view: bool,

    /// Print version information
    #[arg(long = "version")]
    version: bool,

    /// Bookmark management mode (use: -bm, -bm add <name> [path], -bm remove <name>, -bm list)
    #[arg(long = "bm")]
    bookmark_mode: bool,

    /// All positional arguments (path or bookmark commands)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

/// Open a file in the external editor specified in config
fn open_in_editor(file_path: &str, config: &Config) -> Result<()> {
    let editor = &config.behavior.editor;

    // Use shell to execute editor with proper terminal handling
    // Properly quote the file path to handle spaces and special characters
    let shell_cmd = format!("{} '{}' < /dev/tty > /dev/tty 2> /dev/tty",
                            editor,
                            file_path.replace("'", "'\\''"));

    let _status = Command::new("sh")
        .arg("-c")
        .arg(&shell_cmd)
        .status()?;

    // Don't check exit status - many editors return non-zero codes for normal exit
    // (e.g., when user presses Esc or q)

    Ok(())
}

/// Open a binary file in the external hex editor specified in config
fn open_in_hex_editor(file_path: &str, config: &Config) -> Result<()> {
    let hex_editor = &config.behavior.hex_editor;

    // Use shell to execute hex editor with proper terminal handling
    // Properly quote the file path to handle spaces and special characters
    let shell_cmd = format!("{} '{}' < /dev/tty > /dev/tty 2> /dev/tty",
                            hex_editor,
                            file_path.replace("'", "'\\''"));

    let _status = Command::new("sh")
        .arg("-c")
        .arg(&shell_cmd)
        .status()?;

    // Don't check exit status - many hex viewers return non-zero codes for normal exit
    // (e.g., mcview returns 1 when user presses Esc or q)

    Ok(())
}

/// Open a directory in the external file manager specified in config
fn open_in_file_manager(dir_path: &str, config: &Config) -> Result<()> {
    let file_manager = &config.behavior.file_manager;

    // Use shell to execute file manager with proper terminal handling
    // Properly quote the directory path to handle spaces and special characters
    let shell_cmd = format!("{} '{}' < /dev/tty > /dev/tty 2> /dev/tty",
                            file_manager,
                            dir_path.replace("'", "'\\''"));

    let _status = Command::new("sh")
        .arg("-c")
        .arg(&shell_cmd)
        .status()?;

    // Don't check exit status - many file managers return non-zero codes for normal exit
    // (e.g., mc returns 1 when user presses Esc or q)

    Ok(())
}

/// Resolve path or bookmark name to a PathBuf
fn resolve_path_or_bookmark(input: &str, bookmarks: &Bookmarks) -> Result<PathBuf> {
    // 1. If starts with . or / or contains / → treat as path
    if input.starts_with('.') || input.starts_with('/') || input.contains('/') {
        let path = PathBuf::from(input);
        if !path.exists() {
            anyhow::bail!("Directory not found: {}", input);
        }
        return Ok(path.canonicalize()?);
    }

    // 2. Check if it's a bookmark
    if let Some(bookmark) = bookmarks.get(input) {
        if bookmark.path.exists() {
            return Ok(bookmark.path.clone());
        } else {
            anyhow::bail!(
                "Bookmark '{}' points to non-existent directory: {}\n\
                Use 'dt -bm list' to see all bookmarks",
                input, bookmark.path.display()
            );
        }
    }

    // 3. Try as path
    let path = PathBuf::from(input);
    if path.exists() {
        return Ok(path.canonicalize()?);
    }

    // 4. Neither bookmark nor path found
    anyhow::bail!(
        "Neither bookmark '{}' nor directory '{}' found.\n\
        Use 'dt -bm list' to see all bookmarks",
        input, input
    );
}

fn main() -> Result<()> {
    // Preprocess arguments: convert -bm to --bm for clap compatibility
    let args: Vec<String> = std::env::args()
        .map(|arg| if arg == "-bm" { "--bm".to_string() } else { arg })
        .collect();

    // Ensure config file exists (create if missing)
    let config = Config::load();

    let args = Args::parse_from(args);

    // Print version
    if args.version {
        println!("dtree {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // Print help
    if args.help {
        let help_lines = ui::get_help_content();
        for line in help_lines {
            println!("{}", line);
        }
        return Ok(());
    }

    // Handle bookmark management mode
    if args.bookmark_mode {
        let mut bookmarks = Bookmarks::new()?;

        if args.args.is_empty() {
            // Default: list bookmarks
            println!("Bookmarks:");
            if bookmarks.list().is_empty() {
                println!("  No bookmarks saved yet.");
                println!("\nUsage:");
                println!("  dt -bm add <name> [path]    Add a bookmark");
                println!("  dt -bm remove <name>        Remove a bookmark");
                println!("  dt -bm list                 List all bookmarks");
            } else {
                for bookmark in bookmarks.list() {
                    let name = bookmark.name.as_deref().unwrap_or("(unnamed)");
                    println!("  {} → {} ({})", bookmark.key, name, bookmark.path.display());
                }
            }
            return Ok(());
        }

        let subcommand = &args.args[0];
        match subcommand.as_str() {
            "add" => {
                if args.args.len() < 2 {
                    eprintln!("Error: Missing bookmark name");
                    eprintln!("Usage: dt -bm add <name> [path]");
                    std::process::exit(1);
                }
                let name = &args.args[1];
                let path = if args.args.len() >= 3 {
                    PathBuf::from(&args.args[2])
                } else {
                    std::env::current_dir()?
                };

                if !path.exists() {
                    eprintln!("Error: Path does not exist: {}", path.display());
                    std::process::exit(1);
                }

                let mut path = path.canonicalize()?;

                // Bookmarks must be directories only
                if path.is_file() {
                    if let Some(parent) = path.parent() {
                        path = parent.to_path_buf();
                        eprintln!("Note: File provided, using parent directory instead");
                    } else {
                        eprintln!("Error: Cannot determine parent directory");
                        std::process::exit(1);
                    }
                }

                let dir_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string());

                bookmarks.add(name.clone(), path.clone(), dir_name)?;
                println!("✓ Bookmark '{}' added: {}", name, path.display());
            }
            "remove" => {
                if args.args.len() < 2 {
                    eprintln!("Error: Missing bookmark name");
                    eprintln!("Usage: dt -bm remove <name>");
                    std::process::exit(1);
                }
                let name = &args.args[1];
                bookmarks.remove(name)?;
                println!("✓ Bookmark '{}' removed", name);
            }
            "list" => {
                println!("Bookmarks:");
                if bookmarks.list().is_empty() {
                    println!("  No bookmarks saved yet.");
                } else {
                    for bookmark in bookmarks.list() {
                        let name = bookmark.name.as_deref().unwrap_or("(unnamed)");
                        println!("  {} → {} ({})", bookmark.key, name, bookmark.path.display());
                    }
                }
            }
            _ => {
                eprintln!("Error: Unknown bookmark command '{}'", subcommand);
                eprintln!("\nAvailable commands:");
                eprintln!("  dt -bm              List all bookmarks");
                eprintln!("  dt -bm add <name> [path]");
                eprintln!("  dt -bm remove <name>");
                eprintln!("  dt -bm list");
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    // If path or bookmark argument provided, resolve and output without entering TUI
    if !args.args.is_empty() {
        let input = &args.args[0];

        // Special case: -v flag with path/bookmark
        if args.view {
            let bookmarks = Bookmarks::new()?;
            let start_path = resolve_path_or_bookmark(input, &bookmarks)?;

            if !start_path.is_file() {
                eprintln!("Error: --view requires a file path, got: {}", start_path.display());
                std::process::exit(1);
            }

            // Start app in fullscreen viewer mode
            let mut terminal = setup_terminal()?;
            let parent_dir = start_path.parent().unwrap_or(&start_path).to_path_buf();
            let mut app = App::new(parent_dir)?;

            // Set fullscreen mode and load the file
            app.set_fullscreen_viewer(&start_path)?;

            let result = run_app(&mut terminal, &mut app);
            cleanup_terminal()?;

            match result? {
                Some(path) => {
                    let path_str = path.to_string_lossy();
                    if let Some(file_path) = path_str.strip_prefix("EDITOR:") {
                        open_in_editor(file_path, &config)?;
                    } else if let Some(file_path) = path_str.strip_prefix("HEXEDITOR:") {
                        open_in_hex_editor(file_path, &config)?;
                    } else if let Some(dir_path) = path_str.strip_prefix("FILEMGR:") {
                        open_in_file_manager(dir_path, &config)?;
                    } else {
                        println!("{}", path.display());
                    }
                }
                None => {}
            }
            return Ok(());
        }

        // Normal case: resolve path/bookmark and output directly (no TUI)
        let bookmarks = Bookmarks::new()?;
        let resolved_path = resolve_path_or_bookmark(input, &bookmarks)?;

        // Output path for bash wrapper to cd into
        println!("{}", resolved_path.display());
        return Ok(());
    }

    // No arguments: launch interactive TUI from current directory
    let start_path = std::env::current_dir()?;
    let mut terminal = setup_terminal()?;
    let mut app = App::new(start_path)?;
    let result = run_app(&mut terminal, &mut app);

    cleanup_terminal()?;

    match result? {
        Some(path) => {
            let path_str = path.to_string_lossy();
            if let Some(file_path) = path_str.strip_prefix("EDITOR:") {
                open_in_editor(file_path, &config)?;
            } else if let Some(file_path) = path_str.strip_prefix("HEXEDITOR:") {
                open_in_hex_editor(file_path, &config)?;
            } else if let Some(dir_path) = path_str.strip_prefix("FILEMGR:") {
                open_in_file_manager(dir_path, &config)?;
            } else {
                println!("{}", path.display());
            }
        }
        None => {}
    }

    Ok(())
}
