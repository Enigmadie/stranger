use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum ClipboardAction {
    Copy,
    Cut,
}

#[derive(Debug)]
pub enum Clipboard {
    File {
        items: PathBuf,
        action: ClipboardAction,
    },
    Files {
        items: Vec<PathBuf>,
        action: ClipboardAction,
    },
}
