use std::collections::HashMap;
use std::env;
use std::io::{self};
use std::path::{Path, PathBuf};

use ratatui::text::Line;
use tui_textarea::TextArea;

use crate::app::config::constants::model::NUM_COLUMNS;
use crate::app::model::clipboard::Clipboard;
use crate::app::model::miller::columns::MillerColumns;
use crate::app::model::miller::entries::{DirEntry, FileEntry};
use crate::app::model::miller::positions::{
    get_position, parse_path_positions, update_dir_position,
};
use crate::app::model::notification::Notification;
use crate::app::ui::file_preview::PreviewCache;
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
pub mod mark;
pub use mark::Mark;

#[derive(Debug, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Visual { anchor: usize },
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
    pub marked: Vec<PathBuf>,
    pub search_pattern: Option<String>,
    pub show_hidden_files: bool,
    pub preview: Vec<Line<'static>>,
    pub(crate) preview_cache: PreviewCache,
}

impl<'a> State<'a> {
    pub fn new(config: Config) -> io::Result<Self> {
        let current_dir = env::current_dir()?;

        let miller_columns = MillerColumns::build_columns(&current_dir, 0, None, false)?;
        let miller_positions = parse_path_positions(&current_dir, &miller_columns.files);
        let textarea = TextArea::default();

        let mut state = State {
            current_dir,
            files: miller_columns.files,
            dirs: miller_columns.dirs,
            positions_map: miller_positions,
            mode: Mode::Normal,
            modal_type: ModalKind::Disabled,
            input: textarea,
            config,
            from_external_app: false,
            show_hidden_files: false,
            clipboard: None,
            notification: None,
            marked: vec![],
            search_pattern: None,
            preview: Vec::new(),
            preview_cache: PreviewCache::default(),
        };
        let preview_dir = state.current_dir.clone();
        let preview_files = state.files[1].clone();
        state.refresh_preview(&preview_dir, 0, &preview_files);
        Ok(state)
    }

    fn refresh_state(&mut self, target_dir: &Path, new_pos_id: usize) -> io::Result<()> {
        let miller_columns = MillerColumns::build_columns(
            target_dir,
            new_pos_id,
            self.search_pattern.clone(),
            self.show_hidden_files,
        )?;
        let new_pos_id = new_pos_id.min(miller_columns.files[1].len().saturating_sub(1));
        self.refresh_preview(target_dir, new_pos_id, &miller_columns.files[1]);

        self.hide_hint_bar();
        self.current_dir = target_dir.to_path_buf();
        self.files = miller_columns.files;
        self.dirs = miller_columns.dirs;
        update_dir_position(&mut self.positions_map, target_dir, new_pos_id);
        Ok(())
    }

    fn refresh_preview(&mut self, dir: &Path, position_id: usize, files: &[FileEntry]) {
        self.preview = files
            .get(position_id)
            .filter(|file| file.variant.is_regular_file())
            .map(|file| dir.join(&file.name))
            .map(|path| {
                self.preview_cache
                    .load(&path, 2048)
                    .unwrap_or_else(|_| vec![Line::from("Error reading file")])
            })
            .unwrap_or_default();
    }

    pub fn reset_state(&mut self, new_pos_id: usize) -> io::Result<()> {
        self.reset_state_to(self.current_dir.clone(), new_pos_id)
    }

    pub(crate) fn reset_state_to(
        &mut self,
        target_dir: PathBuf,
        new_pos_id: usize,
    ) -> io::Result<()> {
        self.refresh_state(&target_dir, new_pos_id)?;
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
            _ => {
                self.notification = None;
            }
        }
        Ok(())
    }

    pub fn reset_state_except_notifications(&mut self, new_pos_id: usize) -> io::Result<()> {
        let current_dir = self.current_dir.clone();
        self.refresh_state(&current_dir, new_pos_id)?;
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
        let anchor = get_position(&self.positions_map, &self.current_dir)
            .min(self.files[1].len().saturating_sub(1));
        self.marked
            .retain(|path| path.parent() != Some(self.current_dir.as_path()));
        if let Some(file) = self.files[1].get(anchor) {
            self.marked.push(self.current_dir.join(&file.name));
        }
        self.mode = Mode::Visual { anchor };
        self.notification = Notification::Info {
            msg: Lang::en("visual_mode").into(),
        }
        .into();
    }

    fn setup_default_input(&mut self) {
        let textarea = TextArea::default();
        self.input = textarea;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::test_utils::{create_test_state, create_test_state_at};
    use std::fs;
    use tempfile::tempdir;

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

    #[test]
    fn refresh_clamps_a_stale_saved_position() {
        let temp = tempdir().unwrap();
        fs::write(temp.path().join("a"), "").unwrap();
        fs::write(temp.path().join("b"), "").unwrap();
        let mut state = create_test_state_at(temp.path()).unwrap();
        update_dir_position(&mut state.positions_map, &state.current_dir, 10);

        state.reset_state(10).unwrap();

        assert_eq!(get_position(&state.positions_map, &state.current_dir), 1);
        assert_eq!(state.files[1][1].name, "b");
    }
}
