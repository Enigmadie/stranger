use std::collections::HashMap;
use std::env;
use std::io::{self};
use std::path::PathBuf;

use crate::app::model::file_entry::FileEntry;
use crate::app::model::miller::columns::{MillerColumns, NUM_COLUMNS};
use crate::app::model::miller::positions::{
    get_position, parse_path_positions, update_dir_position,
};

#[derive(Debug)]
pub struct State {
    pub current_dir: PathBuf,
    pub exit: bool,
    pub files: [Vec<FileEntry>; NUM_COLUMNS],
    pub dirs: [Option<PathBuf>; NUM_COLUMNS],
    pub positions_map: HashMap<PathBuf, usize>,
    // pub show_popup: bool,
    pub needs_redraw: bool,
}

impl State {
    pub fn new() -> io::Result<Self> {
        let current_dir = env::current_dir()?;

        let miller_columns = MillerColumns::build_columns(&current_dir, 0)?;
        let miller_positions = parse_path_positions(&current_dir);

        Ok(State {
            current_dir,
            files: miller_columns.files,
            dirs: miller_columns.dirs,
            exit: false,
            positions_map: miller_positions,
            // show_popup: false,
            needs_redraw: true,
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
}
