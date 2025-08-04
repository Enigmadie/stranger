use std::path::PathBuf;

#[derive(Debug)]
pub enum ClipboardAction {
    Copy,
    Cut,
}

#[derive(Debug)]
pub enum Clipboard {
    File {
        items: Vec<PathBuf>,
        action: ClipboardAction,
    },
}
