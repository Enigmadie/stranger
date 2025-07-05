use std::env;
use std::io::{self};

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
        let current_dir = env::current_dir()?.display().to_string();

        let files: Vec<String> = std::fs::read_dir(&current_dir)?
            .filter_map(|entry| {
                entry
                    .ok()
                    .map(|e| e.file_name().to_string_lossy().into_owned())
            })
            .collect();

        Ok(State {
            current_dir,
            files,
            exit: false,
            // show_popup: false,
            needs_redraw: true,
        })
    }
}
