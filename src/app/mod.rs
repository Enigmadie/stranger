use crossterm::cursor::Show;
use crossterm::event::DisableMouseCapture;
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::prelude::*;
use std::io::{self, stdout, Stdout};
use std::time::Duration;

pub mod config;
pub mod model;
pub mod state;
pub mod test_utils;
pub mod ui;
pub mod utils;

use crate::app::state::{Modal, Mode, Navigation};
use crate::app::utils::config_parser::load_config;

use self::state::State;

#[derive(Debug)]
pub struct App<'a> {
    state: State<'a>,
    exit: bool,
    needs_redraw: bool,
}

impl<'a> App<'a> {
    pub fn new() -> io::Result<Self> {
        let config = load_config();

        Ok(App {
            state: State::new(config)?,
            exit: false,
            needs_redraw: true,
        })
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
        while !self.exit {
            if event::poll(Duration::from_millis(50))? {
                self.handle_events()?;
            }
            if self.needs_redraw {
                if self.state.from_external_app {
                    terminal.clear()?;
                    self.state.from_external_app = false;
                }
                terminal.draw(|f| ui::render(&self.state, f))?;
                self.needs_redraw = false;
            }
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
                        let _ = self.state.navigate_to_child_or_exec();
                        self.needs_redraw = true;
                    }
                    KeyCode::Char('r') => {
                        self.state.rename();
                        self.needs_redraw = true;
                    }
                    KeyCode::Char('a') => {
                        self.state.add();
                        self.needs_redraw = true;
                    }
                    KeyCode::Char('y') => {
                        self.state.copy_item();
                        self.needs_redraw = true;
                    }
                    KeyCode::Char('p') => {
                        let _ = self.state.paste_item();
                        self.needs_redraw = true;
                    }
                    KeyCode::Char('v') => {
                        self.state.enter_visual_mode();
                        self.needs_redraw = true;
                    }
                    _ => {}
                },
                Mode::Insert => match key.code {
                    KeyCode::Enter => {
                        if self.state.show_popup {
                            self.state.commit();
                        }
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
                Mode::Visual => match key.code {
                    KeyCode::Char('k') | KeyCode::Up => {
                        let _ = self.state.navigate_up();
                        self.needs_redraw = true;
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        let _ = self.state.navigate_down();
                        self.needs_redraw = true;
                    }
                    _ => {}
                },
            }
        }
        Ok(())
    }
}

impl Drop for App<'_> {
    fn drop(&mut self) {
        // Ensure terminal cleanup on drop
        if let Err(e) = cleanup_terminal() {
            eprintln!("Failed to cleanup terminal: {}", e);
        }
    }
}

pub fn cleanup_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    // execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture, Show)?;
    Ok(())
}
