use crossterm::event::{self, Event, KeyCode};
use ratatui::prelude::*;
use std::io::{self, Stdout};

use super::state::State;
use super::ui::{self};

#[derive(Debug)]
pub struct App {
    state: State,
}

impl App {
    pub fn new() -> io::Result<Self> {
        Ok(App {
            state: State::new()?,
        })
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
        while !self.state.exit {
            if self.state.needs_redraw {
                terminal.draw(|f| ui::render(&self.state, f))?;
                self.state.needs_redraw = false;
            }
            let _ = self.handle_events();
        }
        Ok(())
    }

    pub fn handle_events(&mut self) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') {
                self.state.exit = true;
            }
            self.state.needs_redraw = true;
        }
        Ok(())
    }
}
