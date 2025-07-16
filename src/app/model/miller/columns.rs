use std::io::{self};
use std::path::{Path, PathBuf};

use crate::app::config::constants::model::NUM_COLUMNS;
use crate::app::model::file_entry::{FileEntry, FileVariant};

#[derive(Debug)]
pub struct MillerColumns {
    pub dirs: [Option<PathBuf>; NUM_COLUMNS],
    pub files: [Vec<FileEntry>; NUM_COLUMNS],
}

impl MillerColumns {
    pub fn build_columns(current_dir: &Path, position_id: usize) -> io::Result<Self> {
        let selected_dir_files = Self::parse_dir_files(Some(current_dir))?;

        let parent_dir: Option<&Path> = current_dir.parent();

        let parent_dir_files = Self::parse_dir_files(parent_dir)?;

        let (child_dir, child_dir_files) =
            if let Some(first_entry) = selected_dir_files.get(position_id) {
                if first_entry.variant == FileVariant::Directory {
                    let child_path = current_dir.join(&first_entry.name);
                    let child_files = Self::parse_dir_files(Some(&child_path))?;
                    (Some(child_path), child_files)
                } else {
                    (None, vec![])
                }
            } else {
                (None, vec![])
            };

        Ok(Self {
            files: [parent_dir_files, selected_dir_files, child_dir_files],
            dirs: [
                parent_dir.map(|e| e.to_path_buf()),
                Some(current_dir.to_path_buf()),
                child_dir,
            ],
        })
    }

    fn parse_dir_files(current_dir: Option<&Path>) -> io::Result<Vec<FileEntry>> {
        match current_dir {
            Some(dir) => {
                let mut entries: Vec<FileEntry> = std::fs::read_dir(dir)?
                    .filter_map(|entry| {
                        let e = entry.ok()?;
                        let metadata = e.metadata().ok()?;
                        let variant = if metadata.is_dir() {
                            FileVariant::Directory
                        } else {
                            FileVariant::File
                        };

                        Some(FileEntry {
                            name: e.file_name().to_string_lossy().into_owned(),
                            variant,
                        })
                    })
                    .collect();

                entries.sort_by(|a, b| {
                    match (
                        a.variant == FileVariant::Directory,
                        b.variant == FileVariant::Directory,
                    ) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                    }
                });

                Ok(entries)
            }
            None => Ok(vec![]),
        }
    }

    pub fn check_is_current_dir_is_not_empty(files: &[FileEntry]) -> bool {
        !files.is_empty()
    }
}
