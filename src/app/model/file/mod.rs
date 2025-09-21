use std::{
    collections::HashMap,
    fs::Metadata,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Local};
use clap::Error;

use crate::app::{
    model::miller::{
        entries::{FileEntry, FileVariant},
        positions::get_position,
    },
    utils::permissions_to_string,
};

pub fn get_current_file<'a>(
    positions: &HashMap<PathBuf, usize>,
    dir: &PathBuf,
    files: &'a [FileEntry],
) -> Option<&'a FileEntry> {
    let position_id = get_position(positions, dir);
    files.get(position_id)
}

pub fn build_full_path(dir: &Path, file: &FileEntry) -> PathBuf {
    dir.join(&file.name)
}

pub fn calculate_file_size(file_metadata: Metadata) -> u64 {
    file_metadata.len() // in bytes
}

pub fn get_file_permissions(file_metadata: &Metadata) -> String {
    permissions_to_string(&file_metadata.permissions())
}

pub fn get_last_modified(file_metadata: &Metadata) -> Result<String, Error> {
    let modified_time = file_metadata.modified()?;

    let datetime: DateTime<Local> = modified_time.into();
    let formatted_time = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
    Ok(formatted_time)
}

pub fn count_dir_entries<P: AsRef<Path>>(path: P) -> u64 {
    if let Ok(path) = std::fs::read_dir(path) {
        let count = path.count();
        u64::try_from(count).unwrap_or(0)
    } else {
        0
    }
}

pub fn count_matched_files(files: &[FileEntry]) -> usize {
    files
        .iter()
        .filter(|f| match f.variant {
            FileVariant::Directory { is_matched, .. } => is_matched,
            FileVariant::File { is_matched, .. } => is_matched,
        })
        .count()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::app::model::miller::entries::{FileEntry, FileVariant};

    use super::*;

    #[test]
    fn full_path() {
        let dir = PathBuf::from("/src/ui/tests");
        let file = FileEntry {
            name: "test".to_string(),
            variant: FileVariant::File {
                size: Some(10),
                permissions: None,
                last_modified: Some("2023-10-01 12:00".into()),
                is_matched: false,
            },
        };
        let path = build_full_path(&dir, &file);

        assert_eq!(PathBuf::from("/src/ui/tests/test"), path);
    }

    #[test]
    fn current_file() {
        let dir = PathBuf::from("/src/ui/tests");
        let files = vec![FileEntry {
            name: "test".to_string(),
            variant: FileVariant::File {
                size: Some(10),
                permissions: None,
                last_modified: Some("2023-10-01 12:00".into()),
                is_matched: false,
            },
        }];
        let mut positions: HashMap<PathBuf, usize> = HashMap::new();
        positions.insert(dir.clone(), 0);
        let current_file = get_current_file(&positions, &dir, &files);

        assert_eq!(
            Some(&FileEntry {
                name: "test".to_string(),
                variant: FileVariant::File {
                    size: Some(10),
                    permissions: None,
                    last_modified: Some("2023-10-01 12:00".into()),
                    is_matched: false,
                },
            }),
            current_file,
        );
    }
}
