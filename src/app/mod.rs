use ratatui::crossterm::event::{self, Event, KeyCode, KeyModifiers};
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
pub struct App<'a> {
    state: State<'a>,
    exit: bool,
    needs_redraw: bool,
}

impl<'a> App<'a> {
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
        let event = event::read()?;
        if let Event::Key(key) = event {
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
                    KeyCode::Enter => {
                        self.state.push_message();
                        self.needs_redraw = true;
                    }
                    KeyCode::Esc => {
                        self.state.stop_editing();
                        self.needs_redraw = true;
                    }
                    KeyCode::Char('[') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.state.stop_editing();
                        self.needs_redraw = true;
                    }
                    _ => {
                        if self.state.input.lines().join("").len() < 255 {
                            self.state.input.input(event);
                            self.needs_redraw = true;
                        }
                    }
                },
                Mode::Visual => {}
            }
        }
        Ok(())
    }
}
