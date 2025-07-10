use std::env;
use std::io::{self};
use std::path::PathBuf;

use crate::app::model::file_entry::{FileEntry, FileVariant};
use crate::app::model::miller::MillerColumns;

#[derive(Debug)]
pub struct State {
    pub current_dir: PathBuf,
    pub position_id: usize,
    pub exit: bool,
    pub files: [Vec<FileEntry>; 3],
    pub dirs: [Option<PathBuf>; 3],
    // pub show_popup: bool,
    pub needs_redraw: bool,
}

impl State {
    pub fn new() -> io::Result<Self> {
        let current_dir = env::current_dir()?;

        let miller_columns = MillerColumns::build_columns(&current_dir)?;

        // let miller_columns = HashMap::new();
        // let miller_columns_tree = HashMap::new();

        Ok(State {
            current_dir,
            files: miller_columns.files,
            dirs: miller_columns.dirs,
            exit: false,
            position_id: 0,
            // show_popup: false,
            needs_redraw: true,
        })
    }

    pub fn navigate_up(&mut self) -> io::Result<()> {
        if let Some(parent) = &self.dirs[0] {
            self.current_dir = parent.to_path_buf();
            let miller_columns = MillerColumns::build_columns(&self.current_dir)?;
            self.files = miller_columns.files;
            self.dirs = miller_columns.dirs;
        }
        Ok(())
    }

    pub fn navigate_down(&mut self) -> io::Result<()> {
        if let Some(child) = &self.dirs[2] {
            self.current_dir = child.to_path_buf();
            let miller_columns = MillerColumns::build_columns(&self.current_dir)?;
            self.files = miller_columns.files;
            self.dirs = miller_columns.dirs;
        }
        Ok(())
    }
}
