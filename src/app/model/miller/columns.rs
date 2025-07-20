use std::io::{self};
use std::path::{Path, PathBuf};

use crate::app::config::constants::model::NUM_COLUMNS;
use crate::app::model::file::{calculate_file_size, count_dir_entries};
use crate::app::model::miller::entries::{DirEntry, FileEntry, FileVariant};

#[derive(Debug)]
pub struct MillerColumns {
    pub dirs: [DirEntry; NUM_COLUMNS],
    pub files: [Vec<FileEntry>; NUM_COLUMNS],
}

impl MillerColumns {
    pub fn build_columns(current_dir: &Path, position_id: usize) -> io::Result<Self> {
        let selected_dir_entry = DirEntry {
            dir_name: Some(current_dir.to_path_buf()),
            with_meta: true,
        };
        let selected_dir_files = Self::parse_dir_files(&selected_dir_entry)?;

        let parent_dir_entry = DirEntry {
            dir_name: current_dir.parent().map(|e| e.to_path_buf()),
            with_meta: false,
        };

        let parent_dir_files = Self::parse_dir_files(&parent_dir_entry)?;

        let (child_dir_entry, child_dir_files) =
            if let Some(first_entry) = selected_dir_files.get(position_id) {
                if matches!(first_entry.variant, FileVariant::Directory { .. }) {
                    let child_dir_entry = DirEntry {
                        dir_name: Some(current_dir.join(&first_entry.name)),
                        with_meta: true,
                    };
                    let child_files = Self::parse_dir_files(&child_dir_entry)?;
                    (child_dir_entry, child_files)
                } else {
                    (DirEntry::empty_dir(), vec![])
                }
            } else {
                (DirEntry::empty_dir(), vec![])
            };

        Ok(Self {
            files: [parent_dir_files, selected_dir_files, child_dir_files],
            dirs: [parent_dir_entry, selected_dir_entry, child_dir_entry],
        })
    }

    fn parse_dir_files(dir_entry: &DirEntry) -> io::Result<Vec<FileEntry>> {
        match &dir_entry.dir_name {
            Some(dir) => {
                let mut entries: Vec<FileEntry> = std::fs::read_dir(dir)?
                    .filter_map(|entry| {
                        let e = entry.ok()?;
                        let metadata = e.metadata().ok()?;
                        let variant = if metadata.is_dir() {
                            let len = if dir_entry.with_meta {
                                Some(count_dir_entries(e.path()))
                            } else {
                                None
                            };
                            FileVariant::Directory { len }
                        } else {
                            let size = if dir_entry.with_meta {
                                Some(calculate_file_size(metadata))
                            } else {
                                None
                            };
                            FileVariant::File { size }
                        };

                        Some(FileEntry {
                            name: e.file_name().to_string_lossy().into_owned(),
                            variant,
                        })
                    })
                    .collect();

                entries.sort_by(|a, b| {
                    match (
                        matches!(a.variant, FileVariant::Directory { .. }),
                        matches!(b.variant, FileVariant::Directory { .. }),
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
