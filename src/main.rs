use crossterm::{
    event::EnableMouseCapture,
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use ratatui::prelude::*;
use std::io::{self, stdout};

use stranger::{app::cleanup_terminal, App};

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new().unwrap();
    app.run(&mut terminal)?;

    cleanup_terminal()?;

    Ok(())
}
