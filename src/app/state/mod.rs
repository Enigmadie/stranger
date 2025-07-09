use std::env;
use std::io::{self};
use std::path::PathBuf;

use crate::app::model::file_entry::{FileEntry, FileVariant};
use crate::app::model::miller::build_miller_columns;

#[derive(Debug)]
pub struct State {
    pub current_dir: PathBuf,
    pub position_id: usize,
    pub exit: bool,
    pub files: [Vec<FileEntry>; 3],
    // pub show_popup: bool,
    pub needs_redraw: bool,
}

impl State {
    pub fn new() -> io::Result<Self> {
        let current_dir = env::current_dir()?;

        let files = build_miller_columns(&current_dir)?;

        // let miller_columns = HashMap::new();
        // let miller_columns_tree = HashMap::new();

        Ok(State {
            current_dir,
            files,
            exit: false,
            position_id: 0,
            // show_popup: false,
            needs_redraw: true,
        })
    }

    pub fn navigate_up(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.files = build_miller_columns(&self.current_dir).unwrap();
        }
    }

    pub fn navigate_down(&mut self) {
        let child_files = &self.files[2];
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.files = build_miller_columns(&self.current_dir).unwrap();
        }
    }
}
