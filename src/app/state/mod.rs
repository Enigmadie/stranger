use std::collections::HashMap;
use std::env;
use std::io::{self};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct State {
    pub current_dir: String,
    pub position_id: usize,
    pub exit: bool,
    pub files: [Vec<String>; 3],
    // pub show_popup: bool,
    pub needs_redraw: bool,
}

impl State {
    pub fn new() -> io::Result<Self> {
        let current_dir = env::current_dir()?;

        let files = Self::parse_miller_columns(&current_dir)?;

        // let miller_columns = HashMap::new();
        // let miller_columns_tree = HashMap::new();

        Ok(State {
            current_dir: current_dir.display().to_string(),
            files,
            exit: false,
            position_id: 0,
            // show_popup: false,
            needs_redraw: true,
        })
    }

    fn parse_miller_columns(current_dir: &PathBuf) -> io::Result<[Vec<String>; 3]> {
        let selected_dir_files: Vec<String> = std::fs::read_dir(current_dir)?
            .filter_map(|entry| {
                entry
                    .ok()
                    .map(|e| e.file_name().to_string_lossy().into_owned())
            })
            .collect();

        let parent_dir: Option<&Path> = current_dir.parent();

        let parent_dir_files: Vec<String> = match parent_dir {
            Some(dir) => std::fs::read_dir(dir)?
                .filter_map(|entry| {
                    entry
                        .ok()
                        .map(|e| e.file_name().to_string_lossy().into_owned())
                })
                .collect(),
            None => vec![],
        };

        // let mut millerColumnsMap: HashMap<PathBuf, usize> = HashMap::new();
        //
        // millerColumnsMap
        //     .entry(current_dir.to_path_buf())
        //     .and_modify(|e| *e = 0) //temp
        //     .or_insert(0);
        //
        // if let Some(parent) = parent_dir {
        //     millerColumnsMap
        //         .entry(parent.to_path_buf())
        //         .and_modify(|e| *e = 0) //temp
        //         .or_insert(0);
        // }

        // let child_dir: Option
        Ok([parent_dir_files, selected_dir_files, vec![]])
    }
}
