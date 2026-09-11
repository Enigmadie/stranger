use crossterm::{
    event::EnableMouseCapture,
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use ratatui::prelude::*;
use std::io::{self, stdout, Stdout};

use stranger::{
    app::{cleanup_terminal, utils::config_parser::load_config},
    App,
};

type TuiTerminal = Terminal<CrosstermBackend<Stdout>>;

struct TerminalGuard {
    terminal: Option<TuiTerminal>,
}

impl TerminalGuard {
    fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut guard = Self { terminal: None };
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        guard.terminal = Some(Terminal::new(CrosstermBackend::new(stdout))?);
        Ok(guard)
    }

    fn terminal_mut(&mut self) -> &mut TuiTerminal {
        self.terminal
            .as_mut()
            .expect("terminal guard must contain an initialized terminal")
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        if let Err(error) = cleanup_terminal() {
            eprintln!("Failed to cleanup terminal: {error}");
        }
    }
}

fn main() -> io::Result<()> {
    let config = load_config();
    let mut app = App::new(config)?;
    let mut terminal = TerminalGuard::new()?;

    let default_panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = cleanup_terminal();
        default_panic_hook(info);
    }));

    app.run(terminal.terminal_mut())
}
