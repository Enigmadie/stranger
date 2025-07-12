use crossterm::event::{self, Event, KeyCode};
use ratatui::prelude::*;
use std::io::{self, Stdout};

pub mod model;
pub mod state;
pub mod ui;

use crate::app::state::Mode;

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
            match self.state.mode {
                Mode::Normal => match key.code {
                    KeyCode::Char('q') => {
                        self.state.exit = true;
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        let _ = self.state.navigate_up();
                        self.state.needs_redraw = true;
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        let _ = self.state.navigate_down();
                        self.state.needs_redraw = true;
                    }
                    KeyCode::Char('h') | KeyCode::Left => {
                        let _ = self.state.navigate_to_parent();
                        self.state.needs_redraw = true;
                    }
                    KeyCode::Char('l') | KeyCode::Right => {
                        let _ = self.state.navigate_to_child();
                        self.state.needs_redraw = true;
                    }
                    KeyCode::Char('r') => {
                        self.state.rename();
                        self.state.needs_redraw = true;
                    }
                    _ => {}
                },
                Mode::Insert => match key.code {
                    KeyCode::Char(c) => {
                        self.state.input.push(c);
                        self.state.needs_redraw = true;
                    }
                    KeyCode::Enter => {
                        self.state.needs_redraw = true;
                    }
                    _ => {}
                },
                Mode::Visual => {}
            }
        }
        Ok(())
    }
}
