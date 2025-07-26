use std::collections::HashMap;
use std::env;
use std::io::{self};
use std::path::PathBuf;

use tui_textarea::TextArea;

use crate::app::config::constants::model::NUM_COLUMNS;
use crate::app::model::clipboard::Clipboard;
use crate::app::model::file::{build_full_path, get_current_file};
use crate::app::model::miller::columns::MillerColumns;
use crate::app::model::miller::entries::{DirEntry, FileEntry, FileVariant};
use crate::app::model::miller::positions::{get_position, parse_path_positions};
use crate::app::ui::modal::{ModalKind, UnderLineModalAction};
use crate::app::utils::config_parser::default_config::Config;
use crate::app::utils::fs::{copy_file_path, exec};
pub mod modal;
pub use modal::Modal;
pub mod navigation;
pub use navigation::Navigation;

#[derive(Debug)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
}

#[derive(Debug)]
pub struct State<'a> {
    pub current_dir: PathBuf,
    pub files: [Vec<FileEntry>; NUM_COLUMNS],
    pub dirs: [DirEntry; NUM_COLUMNS],
    pub positions_map: HashMap<PathBuf, usize>,
    pub mode: Mode,
    pub show_popup: bool,
    pub modal_type: ModalKind,
    pub input: TextArea<'a>,
    pub err_msg: Option<String>, // TODO
    pub config: Config,
    pub from_external_app: bool,
    pub clipboard: Option<Clipboard>,
}

impl<'a> State<'a> {
    pub fn new(config: Config) -> io::Result<Self> {
        let current_dir = env::current_dir()?;

        let miller_columns = MillerColumns::build_columns(&current_dir, 0)?;
        let miller_positions = parse_path_positions(&current_dir);
        let textarea = TextArea::default();

        Ok(State {
            current_dir,
            files: miller_columns.files,
            dirs: miller_columns.dirs,
            positions_map: miller_positions,
            mode: Mode::Normal,
            show_popup: false,
            modal_type: ModalKind::UnderLine {
                action: UnderLineModalAction::Add,
            },
            input: textarea,
            err_msg: None,
            config,
            from_external_app: false,
            clipboard: None,
            // notification state
        })
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

    pub fn execute_file(&mut self, file_name: PathBuf) {
        let _ = exec(&self.config.common.editor, &[&file_name.to_string_lossy()]);
        self.from_external_app = true;
    }

    pub fn refresh(&mut self, new_pos_id: usize) -> io::Result<()> {
        let miller_columns = MillerColumns::build_columns(&self.current_dir, new_pos_id)?;
        self.files = miller_columns.files;
        self.dirs = miller_columns.dirs;
        self.err_msg = None;
        Ok(())
    }

    pub fn stop_editing(&mut self) {
        self.mode = Mode::Normal;
        self.show_popup = false;
    }

    fn start_editing(&mut self) {
        self.mode = Mode::Insert;
        self.show_popup = true;
    }

    fn setup_default_input(&mut self) {
        let textarea = TextArea::default();
        self.input = textarea;
    }

    pub fn copy_file(&mut self) {
        let current_file = get_current_file(&self.positions_map, &self.current_dir, &self.files[1]);
        if let Some(file) = current_file {
            let file_path = build_full_path(&self.current_dir, file);
            let copied_file = copy_file_path(file_path);
        }
    }
}
