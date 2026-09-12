use std::io::{self};
use std::path::Path;

use crate::app::config::constants::model::NUM_COLUMNS;
use crate::app::model::file::{
    calculate_file_size, count_dir_entries, get_file_permissions, get_last_modified,
};
use crate::app::model::miller::entries::{DirEntry, FileEntry, FileVariant};

#[derive(Debug)]
pub struct MillerColumns {
    pub dirs: [DirEntry; NUM_COLUMNS],
    pub files: [Vec<FileEntry>; NUM_COLUMNS],
}

impl MillerColumns {
    pub fn build_columns(
        current_dir: &Path,
        position_id: usize,
        search_pattern: Option<String>,
        show_hidden_files: bool,
    ) -> io::Result<Self> {
        let selected_dir_entry = DirEntry {
            dir_name: Some(current_dir.to_path_buf()),
            with_meta: true,
        };
        let selected_dir_files =
            Self::parse_dir_files(&selected_dir_entry, &search_pattern, show_hidden_files)?;

        let parent_dir_entry = DirEntry {
            dir_name: current_dir.parent().map(|e| e.to_path_buf()),
            with_meta: false,
        };

        let parent_dir_files =
            Self::parse_dir_files(&parent_dir_entry, &search_pattern, show_hidden_files)?;

        let (child_dir_entry, child_dir_files) = if let Some(first_entry) =
            selected_dir_files.get(position_id)
        {
            if first_entry.variant.is_directory() {
                let child_dir_entry = DirEntry {
                    dir_name: Some(current_dir.join(&first_entry.name)),
                    with_meta: true,
                };
                let child_files =
                    Self::parse_child_preview(&child_dir_entry, &search_pattern, show_hidden_files);
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

    fn parse_dir_files(
        dir_entry: &DirEntry,
        search_pattern: &Option<String>,
        show_hidden_files: bool,
    ) -> io::Result<Vec<FileEntry>> {
        match &dir_entry.dir_name {
            Some(dir) => {
                let mut entries: Vec<FileEntry> = std::fs::read_dir(dir)?
                    .filter_map(|entry| {
                        let e = entry.ok()?;
                        let metadata = e.path().symlink_metadata().ok()?;
                        let permissions =
                            dir_entry.with_meta.then(|| get_file_permissions(&metadata));
                        let last_modified = dir_entry
                            .with_meta
                            .then(|| get_last_modified(&metadata).unwrap_or(String::from("")));
                        let name = e.file_name();
                        let display_name = name.to_string_lossy().into_owned();

                        let is_matched = search_pattern.as_ref().is_some_and(|pattern| {
                            display_name.to_lowercase().starts_with(pattern)
                        });

                        if !show_hidden_files && display_name.starts_with('.') {
                            return None;
                        }

                        let file_type = metadata.file_type();
                        let variant = if file_type.is_symlink() {
                            FileVariant::Symlink { is_matched }
                        } else if file_type.is_dir() {
                            let len = dir_entry.with_meta.then(|| count_dir_entries(e.path()));
                            FileVariant::Directory {
                                len,
                                permissions,
                                last_modified,
                                is_matched,
                            }
                        } else if file_type.is_file() {
                            let size = dir_entry.with_meta.then(|| calculate_file_size(metadata));
                            FileVariant::File {
                                size,
                                permissions,
                                last_modified,
                                is_matched,
                            }
                        } else {
                            FileVariant::Special { is_matched }
                        };

                        Some(FileEntry {
                            name,
                            display_name,
                            variant,
                        })
                    })
                    .collect();

                entries.sort_by(|a, b| {
                    match (a.variant.is_directory(), b.variant.is_directory()) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => a
                            .display_name
                            .to_lowercase()
                            .cmp(&b.display_name.to_lowercase()),
                    }
                });

                Ok(entries)
            }
            None => Ok(vec![]),
        }
    }

    fn parse_child_preview(
        dir_entry: &DirEntry,
        search_pattern: &Option<String>,
        show_hidden_files: bool,
    ) -> Vec<FileEntry> {
        Self::parse_dir_files(dir_entry, search_pattern, show_hidden_files).unwrap_or_default()
    }

    pub fn check_is_current_dir_is_not_empty(files: &[FileEntry]) -> bool {
        !files.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[cfg(target_os = "linux")]
    #[test]
    fn preserves_non_utf8_names_for_filesystem_operations() {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt};

        let temp = tempdir().unwrap();
        let name = OsString::from_vec(vec![b'n', 0xff]);
        fs::write(temp.path().join(&name), "content").unwrap();

        let columns = MillerColumns::build_columns(temp.path(), 0, None, true).unwrap();
        let entry = columns.files[1]
            .iter()
            .find(|entry| entry.name == name)
            .unwrap();

        assert_eq!(entry.name, name);
        assert!(entry.display_name.contains('\u{fffd}'));
    }

    #[cfg(unix)]
    #[test]
    fn distinguishes_symlinks_and_special_files() {
        use std::{ffi::CString, os::unix::ffi::OsStrExt};

        let temp = tempdir().unwrap();
        fs::write(temp.path().join("target"), "content").unwrap();
        std::os::unix::fs::symlink("target", temp.path().join("link")).unwrap();
        let fifo = temp.path().join("pipe");
        let fifo_c = CString::new(fifo.as_os_str().as_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(fifo_c.as_ptr(), 0o600) }, 0);

        let columns = MillerColumns::build_columns(temp.path(), 0, None, true).unwrap();
        let link = columns.files[1]
            .iter()
            .find(|entry| entry.name == "link")
            .unwrap();
        let pipe = columns.files[1]
            .iter()
            .find(|entry| entry.name == "pipe")
            .unwrap();

        assert!(matches!(link.variant, FileVariant::Symlink { .. }));
        assert!(matches!(pipe.variant, FileVariant::Special { .. }));
    }

    #[test]
    fn child_preview_failure_is_non_fatal() {
        let missing = DirEntry {
            dir_name: Some(Path::new("/definitely/missing/stranger-preview").to_path_buf()),
            with_meta: true,
        };

        assert!(MillerColumns::parse_child_preview(&missing, &None, false).is_empty());
    }
}
