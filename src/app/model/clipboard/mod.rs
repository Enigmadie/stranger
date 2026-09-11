use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardAction {
    Copy,
    Cut,
    Delete,
}

#[derive(Debug)]
pub enum Clipboard {
    File {
        items: Vec<PathBuf>,
        action: ClipboardAction,
    },
}
