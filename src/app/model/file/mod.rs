use std::{
    collections::HashMap,
    fs::Metadata,
    path::{Path, PathBuf},
};

use crate::app::model::miller::positions::get_position;

#[derive(Debug, PartialEq)]
pub enum FileVariant {
    Directory { len: u64 },
    File { size: u64 },
}

#[derive(Debug, PartialEq)]
pub struct FileEntry {
    pub name: String,
    pub variant: FileVariant,
}

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

pub fn count_dir_entries<P: AsRef<Path>>(path: P) -> u64 {
    if let Ok(path) = std::fs::read_dir(path) {
        let count = path.count();
        u64::try_from(count).unwrap_or(0)
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn full_path() {
        let dir = PathBuf::from("/src/ui/tests");
        let file = FileEntry {
            name: "test".to_string(),
            variant: FileVariant::File { size: 10 },
        };
        let path = build_full_path(&dir, &file);

        assert_eq!(PathBuf::from("/src/ui/tests/test"), path);
    }

    #[test]
    fn current_file() {
        let dir = PathBuf::from("/src/ui/tests");
        let files = vec![FileEntry {
            name: "test".to_string(),
            variant: FileVariant::File { size: 10 },
        }];
        let mut positions: HashMap<PathBuf, usize> = HashMap::new();
        positions.insert(dir.clone(), 0);
        let current_file = get_current_file(&positions, &dir, &files);

        assert_eq!(
            Some(&FileEntry {
                name: "test".to_string(),
                variant: FileVariant::File { size: 10 },
            }),
            current_file,
        );
    }
}
