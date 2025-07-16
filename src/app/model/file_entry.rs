use std::{collections::HashMap, io, path::PathBuf};

use crate::app::model::miller::positions::get_position;

#[derive(Debug, PartialEq)]
pub enum FileVariant {
    Directory,
    File,
}

#[derive(Debug, PartialEq)]
pub struct FileEntry {
    pub name: String,
    pub variant: FileVariant,
}

pub fn rename_file(path: &PathBuf, new_value: String) -> io::Result<()> {
    std::fs::rename(path, new_value)
}

pub fn get_current_file<'a>(
    positions: &HashMap<PathBuf, usize>,
    dir: &PathBuf,
    files: &'a [FileEntry],
) -> Option<&'a FileEntry> {
    let position_id = get_position(&positions, &dir);
    files.get(position_id)
}

pub fn build_full_path(dir: &PathBuf, file: &FileEntry) -> PathBuf {
    dir.join(&file.name)
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
            variant: FileVariant::File,
        };
        let path = build_full_path(&dir, &file);

        assert_eq!(PathBuf::from("/src/ui/tests/test"), path);
    }

    #[test]
    fn current_file() {
        let dir = PathBuf::from("/src/ui/tests");
        let files = vec![FileEntry {
            name: "test".to_string(),
            variant: FileVariant::File,
        }];
        let mut positions: HashMap<PathBuf, usize> = HashMap::new();
        positions.insert(dir.clone(), 0);
        let current_file = get_current_file(&positions, &dir, &files);

        assert_eq!(
            Some(&FileEntry {
                name: "test".to_string(),
                variant: FileVariant::File
            }),
            current_file,
        );
    }
}
