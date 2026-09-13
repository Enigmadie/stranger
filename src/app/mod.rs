use crossterm::cursor::Show;
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
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
use crate::app::model::notification::Notification;
use crate::app::state::file_managment::DeleteMode;
use crate::app::state::search::SearchDirection;
use crate::app::state::{Bookmarks, FileManager, HintBar, Mark, Mode, Navigation, Search};

use crate::app::ui::modal::hint_bar::HintBarMode;
use crate::app::ui::modal::ModalKind;
use crate::app::utils::config_parser::default_config::Config;

use self::state::State;

const INPUT_LIMIT_BYTES: usize = 255;

fn accepts_key_event(
    mode: &Mode,
    modal: &ModalKind,
    kind: KeyEventKind,
    code: KeyCode,
    modifiers: KeyModifiers,
) -> bool {
    match kind {
        KeyEventKind::Press => true,
        KeyEventKind::Release => false,
        KeyEventKind::Repeat => {
            if modal.is_hint_bar() {
                return false;
            }
            match mode {
                Mode::Insert => true,
                Mode::Normal | Mode::Search => {
                    matches!(code, KeyCode::Up | KeyCode::Down | KeyCode::Char('j' | 'k'))
                        || modifiers.contains(KeyModifiers::CONTROL)
                            && matches!(code, KeyCode::Char('d' | 'u'))
                        || matches!(mode, Mode::Search) && matches!(code, KeyCode::Char('n' | 'N'))
                }
                Mode::Visual { .. } | Mode::Bookmarks { .. } => {
                    matches!(code, KeyCode::Up | KeyCode::Down | KeyCode::Char('j' | 'k'))
                }
            }
        }
    }
}

fn accepts_input(current_bytes: usize, code: KeyCode, modifiers: KeyModifiers) -> bool {
    let inserted_bytes = match code {
        KeyCode::Char(character)
            if !modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER) =>
        {
            character.len_utf8()
        }
        KeyCode::Tab => 1,
        _ => 0,
    };
    inserted_bytes == 0 || current_bytes.saturating_add(inserted_bytes) <= INPUT_LIMIT_BYTES
}

#[derive(Debug)]
pub struct App<'a> {
    state: State<'a>,
    exit: bool,
    needs_redraw: bool,
}

impl<'a> App<'a> {
    pub fn new(config: Config) -> io::Result<Self> {
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
            if !accepts_key_event(
                &self.state.mode,
                &self.state.modal_type,
                key.kind,
                key.code,
                key.modifiers,
            ) {
                return Ok(());
            }
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
                                    let result = self.state.delete_files(DeleteMode::Trash);
                                    self.report_error(result);
                                    self.needs_redraw = true;
                                }
                                KeyCode::Char('x') => {
                                    let result = self.state.delete_files(DeleteMode::Permanent);
                                    self.report_error(result);
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
                                let result = self.state.navigate_up(1);
                                self.report_error(result);
                                self.needs_redraw = true;
                            }
                            KeyCode::Char('j') | KeyCode::Down => {
                                let result = self.state.navigate_down(1);
                                self.report_error(result);
                                self.needs_redraw = true;
                            }
                            KeyCode::Char('h') | KeyCode::Left => {
                                if key.modifiers.contains(KeyModifiers::CONTROL) {
                                    let result = self.state.toggle_hidden_files();
                                    self.report_error(result);
                                } else {
                                    let result = self.state.navigate_to_parent();
                                    self.report_error(result);
                                }
                                self.needs_redraw = true;
                            }
                            KeyCode::Char('l') | KeyCode::Right => {
                                let result = self.state.navigate_to_child_or_exec();
                                self.report_error(result);
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
                                let result = self.state.paste_files();
                                self.report_error(result);
                                self.needs_redraw = true;
                            }
                            KeyCode::Char('v') => {
                                self.state.enter_visual_mode();
                                self.needs_redraw = true;
                            }
                            KeyCode::Char(' ') => {
                                let result = self.state.mark_and_down();
                                self.report_error(result);
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
                                    let result = self.state.navigate_down(25);
                                    self.report_error(result);
                                } else {
                                    self.state.open_hint_bar(HintBarMode::Delete);
                                }
                                self.needs_redraw = true;
                            }
                            KeyCode::Char('u') => {
                                if key.modifiers.contains(KeyModifiers::CONTROL) {
                                    let result = self.state.navigate_up(25);
                                    self.report_error(result);
                                }
                                self.needs_redraw = true;
                            }
                            KeyCode::Char('/') => {
                                self.state.search();
                                self.needs_redraw = true;
                            }
                            KeyCode::Esc => {
                                let result = self.state.exit_search_mode();
                                self.report_error(result);
                                self.state.clear_marks();
                                self.needs_redraw = true;
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') => {
                                if self.state.mode == Mode::Search {
                                    let direction = if key.code == KeyCode::Char('N') {
                                        SearchDirection::Backward
                                    } else {
                                        SearchDirection::Forward
                                    };
                                    let result = self.state.next_match(direction);
                                    self.report_error(result);
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
                            let result = self.state.commit_search();
                            self.report_error(result);
                        }
                        self.needs_redraw = true;
                    }
                    KeyCode::Esc => {
                        self.state.enter_normal_mode();
                        self.needs_redraw = true;
                    }
                    _ => {
                        let input_bytes = self.state.input.lines().join("").len();
                        if accepts_input(input_bytes, key.code, key.modifiers) {
                            self.state.input.input(event);
                            self.needs_redraw = true;
                        }
                    }
                },
                Mode::Visual { .. } => match key.code {
                    KeyCode::Char('k') | KeyCode::Up => {
                        let result = self.state.navigate_up(1);
                        self.report_error(result);
                        self.needs_redraw = true;
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        let result = self.state.navigate_down(1);
                        self.report_error(result);
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
                        let result = self.state.bookmarks_nagivate_up();
                        self.report_error(result);
                        self.needs_redraw = true;
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        let result = self.state.bookmarks_nagivate_down();
                        self.report_error(result);
                        self.needs_redraw = true;
                    }
                    KeyCode::Char('d') => {
                        let result = self.state.delete_from_bookmarks();
                        self.report_error(result);
                        self.needs_redraw = true;
                    }
                    KeyCode::Char('l') => {
                        let result = self.state.open_dir_from_bookmark();
                        self.report_error(result);
                        self.needs_redraw = true;
                    }
                    KeyCode::Enter => {
                        let result = self.state.open_dir_from_bookmark();
                        self.report_error(result);
                        self.needs_redraw = true;
                    }
                    _ => {}
                },
            }
        }
        Ok(())
    }

    fn report_error(&mut self, result: io::Result<()>) {
        if let Err(error) = result {
            self.state.notification = Some(Notification::Error {
                msg: error.to_string().into(),
            });
        }
    }
}

pub fn cleanup_terminal() -> io::Result<()> {
    let raw_result = disable_raw_mode().map_err(io::Error::other);
    let screen_result = execute!(stdout(), LeaveAlternateScreen);
    let cursor_result = execute!(stdout(), Show);

    raw_result.and(screen_result).and(cursor_result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_errors_are_reported_as_notifications() {
        let mut app = App::new(Config::default()).unwrap();

        app.report_error(Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "denied",
        )));

        assert!(matches!(
            app.state.notification,
            Some(Notification::Error { .. })
        ));
    }

    #[test]
    fn repeat_and_release_cannot_complete_multi_key_commands() {
        let mode = Mode::Normal;
        let modal = ModalKind::HintBar {
            mode: HintBarMode::Delete,
        };

        assert!(!accepts_key_event(
            &mode,
            &modal,
            KeyEventKind::Repeat,
            KeyCode::Char('d'),
            KeyModifiers::NONE,
        ));
        assert!(!accepts_key_event(
            &mode,
            &modal,
            KeyEventKind::Release,
            KeyCode::Char('d'),
            KeyModifiers::NONE,
        ));
        assert!(accepts_key_event(
            &mode,
            &modal,
            KeyEventKind::Press,
            KeyCode::Char('d'),
            KeyModifiers::NONE,
        ));
    }

    #[test]
    fn input_limit_only_blocks_insertion() {
        assert!(!accepts_input(
            INPUT_LIMIT_BYTES,
            KeyCode::Char('a'),
            KeyModifiers::NONE
        ));
        assert!(!accepts_input(
            INPUT_LIMIT_BYTES - 1,
            KeyCode::Char('я'),
            KeyModifiers::NONE
        ));
        assert!(accepts_input(
            INPUT_LIMIT_BYTES,
            KeyCode::Backspace,
            KeyModifiers::NONE
        ));
        assert!(accepts_input(
            INPUT_LIMIT_BYTES,
            KeyCode::Left,
            KeyModifiers::NONE
        ));
        assert!(accepts_input(
            INPUT_LIMIT_BYTES,
            KeyCode::Char('a'),
            KeyModifiers::CONTROL
        ));
    }
}
