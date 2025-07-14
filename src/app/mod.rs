use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::prelude::*;
use std::io::{self, Stdout};

pub mod config;
pub mod model;
pub mod state;
pub mod test_utils;
pub mod ui;

use crate::app::state::Mode;

use self::state::State;

#[derive(Debug)]
pub struct App {
    state: State,
    exit: bool,
    needs_redraw: bool,
}

impl App {
    pub fn new() -> io::Result<Self> {
        Ok(App {
            state: State::new()?,
            exit: false,
            needs_redraw: true,
        })
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
        while !self.exit {
            if self.needs_redraw {
                terminal.draw(|f| ui::render(&self.state, f))?;
                self.needs_redraw = false;
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
                        self.exit = true;
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        let _ = self.state.navigate_up();
                        self.needs_redraw = true;
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        let _ = self.state.navigate_down();
                        self.needs_redraw = true;
                    }
                    KeyCode::Char('h') | KeyCode::Left => {
                        let _ = self.state.navigate_to_parent();
                        self.needs_redraw = true;
                    }
                    KeyCode::Char('l') | KeyCode::Right => {
                        let _ = self.state.navigate_to_child();
                        self.needs_redraw = true;
                    }
                    KeyCode::Char('r') => {
                        self.state.rename();
                        self.needs_redraw = true;
                    }
                    _ => {}
                },
                Mode::Insert => match key.code {
                    KeyCode::Char('q') => {
                        self.exit = true;
                    }
                    KeyCode::Esc => {
                        self.state.exit_insert_mode();
                        self.needs_redraw = true;
                    }
                    KeyCode::Char('[') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.state.exit_insert_mode();
                        self.needs_redraw = true;
                    }
                    KeyCode::Char(c) => {
                        self.state.input.push(c);
                        self.needs_redraw = true;
                    }
                    KeyCode::Enter => {
                        self.needs_redraw = true;
                    }
                    _ => {}
                },
                Mode::Visual => {}
            }
        }
        Ok(())
    }
}
