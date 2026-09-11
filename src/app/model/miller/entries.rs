use std::{ffi::OsString, path::PathBuf};

#[derive(Debug, PartialEq, Clone)]
pub enum FileVariant {
    Directory {
        len: Option<u64>,
        permissions: Option<String>,
        last_modified: Option<String>,
        is_matched: bool,
    },
    File {
        size: Option<u64>,
        permissions: Option<String>,
        last_modified: Option<String>,
        is_matched: bool,
    },
    Symlink {
        is_matched: bool,
    },
    Special {
        is_matched: bool,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct FileEntry {
    pub name: OsString,
    pub display_name: String,
    pub variant: FileVariant,
}

impl FileVariant {
    pub fn is_directory(&self) -> bool {
        matches!(self, Self::Directory { .. })
    }

    pub fn is_regular_file(&self) -> bool {
        matches!(self, Self::File { .. })
    }

    pub fn is_matched(&self) -> bool {
        match self {
            Self::Directory { is_matched, .. }
            | Self::File { is_matched, .. }
            | Self::Symlink { is_matched }
            | Self::Special { is_matched } => *is_matched,
        }
    }
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
