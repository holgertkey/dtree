mod tree_node;
mod app;
mod terminal;

use anyhow::Result;
use app::App;
use terminal::{setup_terminal, cleanup_terminal, run_app};

fn main() -> Result<()> {
    let start_path = std::env::current_dir()?;

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
