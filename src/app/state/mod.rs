use std::collections::HashMap;
use std::env;
use std::io::{self};
use std::path::PathBuf;

use ratatui::style::{Color, Modifier, Style};
use tui_textarea::TextArea;

use crate::app::config::constants::model::NUM_COLUMNS;
use crate::app::model::file_entry::FileEntry;
use crate::app::model::miller::columns::MillerColumns;
use crate::app::model::miller::positions::{
    get_position, parse_path_positions, update_dir_position,
};
use crate::app::ui::modal::ModalKind;

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
}

impl<'a> State<'a> {
    pub fn new() -> io::Result<Self> {
        let current_dir = env::current_dir()?;

        let miller_columns = MillerColumns::build_columns(&current_dir, 0)?;
        let miller_positions = parse_path_positions(&current_dir);
        let textarea = TextArea::default();
        // textarea.set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));

        Ok(State {
            current_dir,
            files: miller_columns.files,
            dirs: miller_columns.dirs,
            positions_map: miller_positions,
            mode: Mode::Normal,
            show_popup: false,
            modal_type: ModalKind::UnderLine,
            input: textarea,
        })
    }

    pub fn navigate_to_parent(&mut self) -> io::Result<()> {
        if let Some(parent) = &self.dirs[0] {
            self.current_dir = parent.to_path_buf();
            let position_id = get_position(&self.positions_map, &self.current_dir);
            let miller_columns = MillerColumns::build_columns(&self.current_dir, position_id)?;
            self.files = miller_columns.files;
            self.dirs = miller_columns.dirs;
        }
        Ok(())
    }

    pub fn navigate_to_child(&mut self) -> io::Result<()> {
        if let Some(child) = &self.dirs[2] {
            self.current_dir = child.to_path_buf();
            let position_id = get_position(&self.positions_map, &self.current_dir);
            let miller_columns = MillerColumns::build_columns(&self.current_dir, position_id)?;
            self.files = miller_columns.files;
            self.dirs = miller_columns.dirs;
        }
        Ok(())
    }

    pub fn navigate_up(&mut self) -> io::Result<()> {
        let position_id = get_position(&self.positions_map, &self.current_dir);
        let new_position_id = position_id.saturating_sub(1);

        update_dir_position(&mut self.positions_map, &self.current_dir, new_position_id);
        let miller_columns = MillerColumns::build_columns(&self.current_dir, new_position_id)?;
        self.files = miller_columns.files;
        self.dirs = miller_columns.dirs;
        Ok(())
    }

    pub fn navigate_down(&mut self) -> io::Result<()> {
        let position_id = get_position(&self.positions_map, &self.current_dir);
        if position_id < self.files[1].len().saturating_sub(1) {
            let new_position_id = position_id + 1;
            update_dir_position(&mut self.positions_map, &self.current_dir, new_position_id);
            let miller_columns = MillerColumns::build_columns(&self.current_dir, new_position_id)?;
            self.files = miller_columns.files;
            self.dirs = miller_columns.dirs;
        }
        Ok(())
    }

    pub fn rename(&mut self) {
        self.start_editing();
        self.modal_type = ModalKind::UnderLine;
    }

    fn start_editing(&mut self) {
        self.mode = Mode::Insert;
        self.show_popup = true;
        self.to_default_input();
    }

    pub fn stop_editing(&mut self) {
        self.to_default_input();
        self.mode = Mode::Normal;
        self.show_popup = false;
    }

    pub fn push_message(&mut self) {
        let input_value = self.input.lines().join("");
        // logic for push
        self.to_default_input();
        self.mode = Mode::Normal;
    }

    fn to_default_input(&mut self) {
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(Style::default());
        self.input = textarea;
    }
}
