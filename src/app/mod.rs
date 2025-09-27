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

use crate::app::model::clipboard::ClipboardAction;
use crate::app::state::file_managment::DeleteMode;
use crate::app::state::{Bookmarks, FileManager, HintBar, Mark, Mode, Navigation, Search};

use crate::app::ui::modal::hint_bar::HintBarMode;
use crate::app::ui::modal::ModalKind;
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
        if let Event::Resize(_, _) = event {
            self.needs_redraw = true;
            return Ok(());
        }
        if let Event::Key(key) = event {
            match self.state.mode {
                Mode::Normal | Mode::Search => {
                    if let ModalKind::HintBar { mode } = &self.state.modal_type {
                        match mode {
                            HintBarMode::Bookmarks => match key.code {
                                KeyCode::Char('b') => {
                                    self.state.enter_bookmarks_mode();
                                    self.needs_redraw = true;
                                }
                                KeyCode::Char('a') => {
                                    self.state.add_to_bookmarks();
                                    self.needs_redraw = true;
                                }
                                KeyCode::Char('q') => {
                                    self.state.hide_hint_bar();
                                    self.needs_redraw = true;
                                }
                                KeyCode::Esc => {
                                    self.state.hide_hint_bar();
                                    self.needs_redraw = true;
                                }
                                _ => {}
                            },
                            HintBarMode::Delete => match key.code {
                                KeyCode::Char('d') => {
                                    self.state.copy_files(ClipboardAction::Cut);
                                    self.needs_redraw = true;
                                }
                                KeyCode::Char('D') => {
                                    self.state.delete_files(DeleteMode::Trash);
                                    self.needs_redraw = true;
                                }
                                KeyCode::Char('x') => {
                                    self.state.delete_files(DeleteMode::Permanent);
                                    self.needs_redraw = true;
                                }
                                KeyCode::Char('q') => {
                                    self.state.hide_hint_bar();
                                    self.needs_redraw = true;
                                }
                                KeyCode::Esc => {
                                    self.state.hide_hint_bar();
                                    self.needs_redraw = true;
                                }
                                _ => {}
                            },
                            HintBarMode::Exit => match key.code {
                                KeyCode::Char('z') | KeyCode::Char('Z') => {
                                    self.state.switch_to_current_dir();
                                    self.exit = true;
                                }
                                KeyCode::Char('q') | KeyCode::Char('Q') => {
                                    self.exit = true;
                                }
                                KeyCode::Esc => {
                                    self.state.hide_hint_bar();
                                    self.needs_redraw = true;
                                }
                                _ => {}
                            },
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') => {
                                self.exit = true;
                            }
                            KeyCode::Char('k') | KeyCode::Up => {
                                let _ = self.state.navigate_up(1);
                                self.needs_redraw = true;
                            }
                            KeyCode::Char('j') | KeyCode::Down => {
                                let _ = self.state.navigate_down(1);
                                self.needs_redraw = true;
                            }
                            KeyCode::Char('h') | KeyCode::Left => {
                                if key.modifiers.contains(KeyModifiers::CONTROL) {
                                    self.state.toggle_hidden_files();
                                } else {
                                    let _ = self.state.navigate_to_parent();
                                }
                                self.needs_redraw = true;
                            }
                            KeyCode::Char('l') | KeyCode::Right => {
                                let _ = self.state.navigate_to_child_or_exec();
                                self.needs_redraw = true;
                            }
                            KeyCode::Char('r') => {
                                self.state.rename_file();
                                self.needs_redraw = true;
                            }
                            KeyCode::Char('a') => {
                                self.state.add_file();
                                self.needs_redraw = true;
                            }
                            KeyCode::Char('y') => {
                                self.state.copy_files(ClipboardAction::Copy);
                                self.needs_redraw = true;
                            }
                            KeyCode::Char('p') => {
                                let _ = self.state.paste_files();
                                self.needs_redraw = true;
                            }
                            KeyCode::Char('v') => {
                                self.state.enter_visual_mode();
                                self.needs_redraw = true;
                            }
                            KeyCode::Char(' ') => {
                                self.state.mark_and_down();
                                self.needs_redraw = true;
                            }
                            KeyCode::Char('b') => {
                                self.state.open_hint_bar(HintBarMode::Bookmarks);
                                self.needs_redraw = true;
                            }
                            KeyCode::Char('Z') | KeyCode::Char('z') => {
                                self.state.open_hint_bar(HintBarMode::Exit);
                                self.needs_redraw = true;
                            }
                            KeyCode::Char('d') => {
                                if key.modifiers.contains(KeyModifiers::CONTROL) {
                                    let _ = self.state.navigate_down(25);
                                } else {
                                    self.state.open_hint_bar(HintBarMode::Delete);
                                }
                                self.needs_redraw = true;
                            }
                            KeyCode::Char('u') => {
                                if key.modifiers.contains(KeyModifiers::CONTROL) {
                                    let _ = self.state.navigate_up(25);
                                }
                                self.needs_redraw = true;
                            }
                            KeyCode::Char('/') => {
                                self.state.search();
                                self.needs_redraw = true;
                            }
                            KeyCode::Esc => {
                                self.state.exit_search_mode();
                                self.needs_redraw = true;
                            }
                            KeyCode::Char('n') => {
                                if self.state.mode == Mode::Search {
                                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                                        self.state.next_match("prev".to_string());
                                    } else {
                                        self.state.next_match("next".to_string());
                                    }
                                    self.needs_redraw = true;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Mode::Insert => match key.code {
                    KeyCode::Enter => {
                        if self.state.modal_type.is_underline() {
                            self.state.commit_changes();
                        }
                        if self.state.modal_type.is_bottom_line() {
                            self.state.commit_search();
                        }
                        self.needs_redraw = true;
                    }
                    KeyCode::Esc => {
                        self.state.enter_normal_mode();
                        self.needs_redraw = true;
                    }
                    _ => {
                        if self.state.input.lines().join("").len() < 255 {
                            self.state.input.input(event);
                            self.needs_redraw = true;
                        }
                    }
                },
                Mode::Visual { .. } => match key.code {
                    KeyCode::Char('k') | KeyCode::Up => {
                        let _ = self.state.navigate_up(1);
                        self.needs_redraw = true;
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        let _ = self.state.navigate_down(1);
                        self.needs_redraw = true;
                    }
                    KeyCode::Char('[') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.state.enter_normal_mode();
                        self.needs_redraw = true;
                    }
                    KeyCode::Esc => {
                        self.state.enter_normal_mode();
                        self.needs_redraw = true;
                    }
                    KeyCode::Char('v') => {
                        self.state.enter_normal_mode();
                        self.needs_redraw = true;
                    }
                    _ => {}
                },
                Mode::Bookmarks { .. } => match key.code {
                    KeyCode::Char('q') => {
                        self.state.enter_normal_mode();
                        self.needs_redraw = true;
                    }
                    KeyCode::Esc => {
                        self.state.enter_normal_mode();
                        self.needs_redraw = true;
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        let _ = self.state.bookmarks_nagivate_up();
                        self.needs_redraw = true;
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        let _ = self.state.bookmarks_nagivate_down();
                        self.needs_redraw = true;
                    }
                    KeyCode::Char('d') => {
                        self.state.delete_from_bookmarks();
                        self.needs_redraw = true;
                    }
                    KeyCode::Char('l') => {
                        let _ = self.state.open_dir_from_bookmark();
                        self.needs_redraw = true;
                    }
                    KeyCode::Enter => {
                        let _ = self.state.open_dir_from_bookmark();
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
        if let Err(e) = cleanup_terminal() {
            eprintln!("Failed to cleanup terminal: {}", e);
        }
    }
}

pub fn cleanup_terminal() -> io::Result<()> {
    disable_raw_mode().map_err(io::Error::other)?;
    execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture, Show)?;
    Ok(())
}
