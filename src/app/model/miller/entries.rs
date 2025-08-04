use std::path::PathBuf;

#[derive(Debug, PartialEq, Clone)]
pub enum FileVariant {
    Directory { len: Option<u64> },
    File { size: Option<u64> },
}

#[derive(Debug, PartialEq, Clone)]
pub struct FileEntry {
    pub name: String,
    pub variant: FileVariant,
}

#[derive(Debug, PartialEq)]
pub struct DirEntry {
    pub dir_name: Option<PathBuf>,
    pub with_meta: bool,
}

impl DirEntry {
    pub fn empty_dir() -> Self {
        DirEntry {
            dir_name: None,
            with_meta: false,
        }
    }
}
