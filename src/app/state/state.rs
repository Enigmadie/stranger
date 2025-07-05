use std::env;
use std::io::{self, Stdout};

#[derive(Debug)]
pub struct State {
    current_dir: String,
    exit: bool,
    show_popup: bool,
}

impl State {
    pub fn new() -> io::Result<Self> {
        let current_dir = match env::current_dir() {
            Ok(path) => path.display().to_string(),
            Err(_) => "Unkwnown dir".to_string(),
        };

        let files = std::fs::read_dir(&current_dir)?;

        println!("{:?}", files);

        Ok(State {
            current_dir: current_dir,
            exit: false,
            show_popup: false,
        })
    }
}
