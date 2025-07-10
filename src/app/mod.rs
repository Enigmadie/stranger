use crossterm::event::{self, Event, KeyCode};
use ratatui::prelude::*;
use std::io::{self, Stdout};

pub mod model;
pub mod state;
pub mod ui;

use self::state::State;

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
            match key.code {
                KeyCode::Char('q') => {
                    self.state.exit = true;
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.state.position_id = self.state.position_id.saturating_sub(1);
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    if self.state.position_id < self.state.files[1].len().saturating_sub(1) {
                        self.state.position_id += 1;
                    }
                }
                KeyCode::Char('h') | KeyCode::Left => {
                    let _ = self.state.navigate_up();
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    let _ = self.state.navigate_down();
                }
                _ => {}
            }
            self.state.needs_redraw = true;
        }
        Ok(())
    }
}
