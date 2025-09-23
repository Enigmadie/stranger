use crossterm::{
    event::EnableMouseCapture,
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use ratatui::prelude::*;
use std::io::{self, stdout};

use stranger::{app::cleanup_terminal, App};

fn main() -> io::Result<()> {
    std::panic::set_hook(Box::new(|info| {
        let _ = cleanup_terminal();
        eprintln!("panic: {info}");
    }));

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;
    let run_res = app.run(&mut terminal);

    let _ = cleanup_terminal();

    run_res
}
