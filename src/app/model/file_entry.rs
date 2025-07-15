use std::{io, path::PathBuf};

#[derive(Debug, PartialEq)]
pub enum FileVariant {
    Directory,
    File,
}

#[derive(Debug)]
pub struct FileEntry {
    pub name: String,
    pub variant: FileVariant,
}

pub fn rename_file(path: PathBuf, new_value: String) -> io::Result<()> {
    std::fs::rename(path, new_value)
}
