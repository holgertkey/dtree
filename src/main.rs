mod tree_node;
mod app;
mod terminal;
mod navigation;
mod file_viewer;
mod search;
mod ui;
mod event_handler;
mod config;
mod bookmarks;

use anyhow::Result;
use app::App;
use terminal::{setup_terminal, cleanup_terminal, run_app};
use clap::Parser;
use std::path::PathBuf;
use std::process::Command;
use config::Config;

#[derive(Parser)]
#[command(name = "dtree")]
#[command(about = "Interactive directory tree navigator")]
#[command(disable_help_flag = true)]
#[command(disable_version_flag = true)]
struct Args {
    /// Starting directory path (defaults to current directory)
    #[arg(value_name = "PATH")]
    path: Option<PathBuf>,

    /// Print help information
    #[arg(short = 'h', long = "help")]
    help: bool,

    /// View file directly in fullscreen mode
    #[arg(short = 'v', long = "view", conflicts_with = "version")]
    view: bool,

    /// Print version information
    #[arg(long = "version")]
    version: bool,
}

/// Open a file in the external editor specified in config
fn open_in_editor(file_path: &str, config: &Config) -> Result<()> {
    let editor = &config.behavior.editor;

    // Check if editor exists
    if !config.editor_exists() {
        eprintln!("Error: Editor '{}' not found in system.", editor);
        eprintln!("Edit the configuration file to choose the editor.");
        std::process::exit(1);
    }

    // Use shell to execute editor with proper terminal handling
    // Properly quote the file path to handle spaces and special characters
    let shell_cmd = format!("{} '{}' < /dev/tty > /dev/tty 2> /dev/tty",
                            editor,
                            file_path.replace("'", "'\\''"));

    let status = Command::new("sh")
        .arg("-c")
        .arg(&shell_cmd)
        .status()?;

    if !status.success() {
        eprintln!("Error: Editor exited with status: {}", status);
        std::process::exit(1);
    }

    Ok(())
}

/// Open a directory in the external file manager specified in config
fn open_in_file_manager(dir_path: &str, config: &Config) -> Result<()> {
    let file_manager = &config.behavior.file_manager;

    // Check if file manager exists
    if !config.file_manager_exists() {
        eprintln!("Error: File manager '{}' not found in system.", file_manager);
        eprintln!("Edit the configuration file to choose the file manager.");
        std::process::exit(1);
    }

    // Use shell to execute file manager with proper terminal handling
    // Properly quote the directory path to handle spaces and special characters
    let shell_cmd = format!("{} '{}' < /dev/tty > /dev/tty 2> /dev/tty",
                            file_manager,
                            dir_path.replace("'", "'\\''"));

    let status = Command::new("sh")
        .arg("-c")
        .arg(&shell_cmd)
        .status()?;

    if !status.success() {
        eprintln!("Error: File manager exited with status: {}", status);
        std::process::exit(1);
    }

    Ok(())
}

fn main() -> Result<()> {
    // Ensure config file exists (create if missing)
    let config = Config::load();

    let args = Args::parse();

    // Если запрошена версия, выводим её
    if args.version {
        println!("dtree {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // If help requested, print full help
    if args.help {
        let help_lines = ui::get_help_content();
        for line in help_lines {
            println!("{}", line);
        }
        return Ok(());
    }

    let start_path = if let Some(path) = args.path {
        path.canonicalize().unwrap_or_else(|_| path)
    } else {
        std::env::current_dir()?
    };

    // If view mode requested, check that the path is a file
    if args.view {
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
                    // Open file in external editor
                    open_in_editor(file_path, &config)?;
                    Ok(())
                } else if let Some(dir_path) = path_str.strip_prefix("FILEMGR:") {
                    // Open directory in file manager
                    open_in_file_manager(dir_path, &config)?;
                    Ok(())
                } else {
                    println!("{}", path.display());
                    Ok(())
                }
            }
            None => Ok(()),
        }
    } else {
        // Normal tree navigation mode
        let mut terminal = setup_terminal()?;
        let mut app = App::new(start_path)?;
        let result = run_app(&mut terminal, &mut app);

        cleanup_terminal()?;

        match result? {
            Some(path) => {
                let path_str = path.to_string_lossy();
                if let Some(file_path) = path_str.strip_prefix("EDITOR:") {
                    // Open file in external editor
                    open_in_editor(file_path, &config)?;
                    Ok(())
                } else if let Some(dir_path) = path_str.strip_prefix("FILEMGR:") {
                    // Open directory in file manager
                    open_in_file_manager(dir_path, &config)?;
                    Ok(())
                } else {
                    println!("{}", path.display());
                    Ok(())
                }
            }
            None => Ok(()),
        }
    }
}
