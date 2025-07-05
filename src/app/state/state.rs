use std::collections::HashMap;
use std::env;
use std::io::{self};
use std::path::PathBuf;

#[derive(Debug)]
pub struct State {
    pub current_dir: String,
    pub exit: bool,
    pub files: Vec<String>,
    // pub show_popup: bool,
    pub needs_redraw: bool,
}

impl State {
    pub fn new() -> io::Result<Self> {
        let current_dir = env::current_dir()?;

        let files = Self::parse_miller_columns(current_dir)?;

        let miller_columns = HashMap::new();
        let miller_columns_tree = HashMap::new();

        Ok(State {
            current_dir: current_dir.display().to_string(),
            files,
            exit: false,
            // show_popup: false,
            needs_redraw: true,
        })
    }

    fn parse_miller_columns(current_dir: PathBuf) -> io::Result<Vec<String>> {
        let selected_dir_files: Vec<String> =
            std::fs::read_dir(&current_dir.display().to_string())?
                .filter_map(|entry| {
                    entry
                        .ok()
                        .map(|e| e.file_name().to_string_lossy().into_owned())
                })
                .collect();

        // let parent_dir = current_dir.parent().map(|path| path).unwrap_or();

        Ok(selected_dir_files)
    }

    fn parse_column_history() {}
}
