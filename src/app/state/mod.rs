use std::collections::HashMap;
use std::env;
use std::io::{self};
use std::path::PathBuf;

use tui_textarea::TextArea;

use crate::app::config::constants::model::NUM_COLUMNS;
use crate::app::model::file_entry::FileEntry;
use crate::app::model::miller::columns::MillerColumns;
use crate::app::model::miller::positions::parse_path_positions;
use crate::app::ui::modal::{ModalKind, UnderLineModalAction};
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
    pub dirs: [Option<PathBuf>; NUM_COLUMNS],
    pub positions_map: HashMap<PathBuf, usize>,
    pub mode: Mode,
    pub show_popup: bool,
    pub modal_type: ModalKind,
    pub input: TextArea<'a>,
    pub err_msg: Option<String>, // TODO
}

impl<'a> State<'a> {
    pub fn new() -> io::Result<Self> {
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
        })
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
}
