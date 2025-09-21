use std::collections::HashMap;
use std::env;
use std::io::{self};
use std::path::PathBuf;

use tui_textarea::TextArea;

use crate::app::config::constants::model::NUM_COLUMNS;
use crate::app::model::clipboard::Clipboard;
use crate::app::model::file::{build_full_path, count_searched_files, get_current_file};
use crate::app::model::miller::columns::MillerColumns;
use crate::app::model::miller::entries::{DirEntry, FileEntry, FileVariant};
use crate::app::model::miller::positions::parse_path_positions;
use crate::app::model::notification::Notification;
use crate::app::ui::modal::ModalKind;
use crate::app::utils::config_parser::default_config::Config;
use crate::app::utils::i18n::Lang;
pub mod file_managment;
pub use file_managment::FileManager;
pub mod bookmarks;
pub use bookmarks::Bookmarks;
pub mod navigation;
pub use navigation::Navigation;
pub mod hint_bar;
pub use hint_bar::HintBar;
pub mod search;
pub use search::Search;

#[derive(Debug, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Visual { init: bool },
    Bookmarks { position_id: usize },
    Search,
}

#[derive(Debug)]
pub struct State<'a> {
    pub current_dir: PathBuf,
    pub files: [Vec<FileEntry>; NUM_COLUMNS],
    pub dirs: [DirEntry; NUM_COLUMNS],
    pub positions_map: HashMap<PathBuf, usize>,
    pub mode: Mode,
    pub modal_type: ModalKind,
    pub input: TextArea<'a>,
    pub config: Config,
    pub from_external_app: bool,
    pub clipboard: Option<Clipboard>,
    pub notification: Option<Notification>,
    pub marked: Vec<FileEntry>,
    pub search_pattern: Option<String>,
}

impl<'a> State<'a> {
    pub fn new(config: Config) -> io::Result<Self> {
        let current_dir = env::current_dir()?;

        let miller_columns = MillerColumns::build_columns(&current_dir, 0, None)?;
        let miller_positions = parse_path_positions(&current_dir);
        let textarea = TextArea::default();

        Ok(State {
            current_dir,
            files: miller_columns.files,
            dirs: miller_columns.dirs,
            positions_map: miller_positions,
            mode: Mode::Normal,
            modal_type: ModalKind::Disabled,
            input: textarea,
            config,
            from_external_app: false,
            clipboard: None,
            notification: None,
            marked: vec![],
            search_pattern: None,
        })
    }

    fn refresh_state(&mut self, new_pos_id: usize) -> io::Result<()> {
        self.hide_hint_bar();
        let miller_columns = MillerColumns::build_columns(
            &self.current_dir,
            new_pos_id,
            self.search_pattern.clone(),
        )?;
        self.files = miller_columns.files;
        self.dirs = miller_columns.dirs;
        Ok(())
    }

    pub fn reset_state(&mut self, new_pos_id: usize) -> io::Result<()> {
        self.refresh_state(new_pos_id)?;
        match self.mode {
            Mode::Insert => {
                self.notification = Notification::Info {
                    msg: Lang::en("insert_mode").into(),
                }
                .into();
            }
            Mode::Visual { .. } => {
                self.notification = Notification::Info {
                    msg: Lang::en("visual_mode").into(),
                }
                .into();
            }
            Mode::Bookmarks { .. } => {
                self.notification = Notification::Info {
                    msg: Lang::en("bookmarks_mode").into(),
                }
                .into();
            }
            Mode::Search => {
                let searched_files_count = count_searched_files(&self.files[1]);
                self.notification = Notification::Info {
                    msg: Lang::en_fmt("search_mode", &[&searched_files_count.to_string()]).into(),
                }
                .into();
            }
            _ => {
                self.notification = None;
            }
        }
        Ok(())
    }

    pub fn reset_state_except_notifications(&mut self, new_pos_id: usize) -> io::Result<()> {
        self.refresh_state(new_pos_id)?;
        Ok(())
    }

    pub fn navigate_to_child_or_exec(&mut self) -> io::Result<()> {
        let current_file = get_current_file(&self.positions_map, &self.current_dir, &self.files[1]);
        if let Some(file) = current_file {
            let file_path = build_full_path(&self.current_dir, file);
            if let FileVariant::File { .. } = file.variant {
                self.execute_file(file_path);
            } else {
                self.navigate_to_child()?;
            }
        }
        Ok(())
    }

    pub fn enter_normal_mode(&mut self) {
        self.mode = Mode::Normal;
        self.modal_type = ModalKind::Disabled;
        self.notification = None;
        self.setup_default_input();
    }

    fn enter_insert_mode(&mut self) {
        self.mode = Mode::Insert;
        self.notification = Notification::Info {
            msg: Lang::en("insert_mode").into(),
        }
        .into();
    }

    pub fn enter_visual_mode(&mut self) {
        self.mark_item();
        self.mode = Mode::Visual { init: true };
        self.notification = Notification::Info {
            msg: Lang::en("visual_mode").into(),
        }
        .into();
    }

    fn setup_default_input(&mut self) {
        let textarea = TextArea::default();
        self.input = textarea;
    }

    pub fn mark_item(&mut self) {
        let current_file = get_current_file(&self.positions_map, &self.current_dir, &self.files[1]);
        if let Some(file) = current_file {
            let found_file = self.marked.iter().any(|f| f.name == file.name);
            if !found_file {
                self.marked.push(file.clone());
            } else {
                self.marked.retain(|e| e.name != file.name);
            }
        }
    }

    pub fn mark_and_down(&mut self) {
        let current_file = get_current_file(&self.positions_map, &self.current_dir, &self.files[1]);
        if let Some(file) = current_file {
            let found_file = &self.marked.iter().any(|f| f.name == file.name);
            if !found_file {
                self.marked.push(file.clone());
            } else {
                self.marked.retain(|e| e.name != file.name);
            }
        }
        let _ = self.navigate_down();
    }

    pub fn clear_marks(&mut self) {
        self.marked.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::test_utils::create_test_state;

    #[test]
    fn mark_item_adds_file() {
        let mut state = create_test_state();
        let initial_length = state.marked.len();
        state.mark_item();
        assert_eq!(state.marked.len(), initial_length + 1);
    }

    #[test]
    fn mark_item_removes_file() {
        let mut state = create_test_state();
        state.mark_item();
        let initial_length = state.marked.len();
        state.mark_item();
        assert_eq!(state.marked.len(), initial_length - 1);
    }

    #[test]
    fn marks_and_moves_down() {
        let mut state = create_test_state();
        let initial_length = state.marked.len();
        state.mark_and_down();
        assert_eq!(state.marked.len(), initial_length + 1);
        assert_eq!(state.mode, Mode::Normal);
    }

    #[test]
    fn normal_mode_changes_state() {
        let mut state = create_test_state();
        state.enter_normal_mode();
        assert_eq!(state.mode, Mode::Normal);
        assert!(state.notification.is_none());
    }

    #[test]
    fn insert_mode_changes_state() {
        let mut state = create_test_state();
        state.enter_insert_mode();
        assert_eq!(state.mode, Mode::Insert);
        assert!(state.notification.is_some());
    }
}
