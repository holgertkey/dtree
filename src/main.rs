mod tree_node;
mod app;
mod terminal;

use anyhow::Result;
use app::App;
use terminal::{setup_terminal, cleanup_terminal, run_app};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "dtree")]
#[command(version)]
#[command(about = "Interactive directory tree navigator")]
#[command(disable_help_flag = true)]
struct Args {
    /// Starting directory path (defaults to current directory)
    #[arg(value_name = "PATH")]
    path: Option<PathBuf>,

    /// Print help information
    #[arg(short = 'h', long = "help")]
    help: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Если запрошен help, выводим полную справку
    if args.help {
        let help_lines = App::get_help_content();
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

    let mut terminal = setup_terminal()?;
    let mut app = App::new(start_path)?;
    let result = run_app(&mut terminal, &mut app);

    cleanup_terminal()?;

    match result? {
        Some(path) => {
            println!("{}", path.display());
            Ok(())
        }
        None => Ok(()),
    }
}
