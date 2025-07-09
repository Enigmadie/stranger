use std::collections::HashMap;
use std::io::{self};
use std::path::{Path, PathBuf};

use crate::app::model::file_entry::{FileEntry, FileVariant};

pub fn build_miller_columns(current_dir: &PathBuf) -> io::Result<[Vec<FileEntry>; 3]> {
    let selected_dir_files = parse_dir_files(Some(current_dir))?;

    let parent_dir: Option<&Path> = current_dir.parent();

    let parent_dir_files = parse_dir_files(parent_dir)?;

    let child_dir_files = if let Some(first_entry) = selected_dir_files.first() {
        if first_entry.variant == FileVariant::Directory {
            let child_path = current_dir.join(&first_entry.name);
            parse_dir_files(Some(&child_path))?
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    // let mut millerColumnsMap: HashMap<PathBuf, usize> = HashMap::new();
    //
    // millerColumnsMap
    //     .entry(current_dir.to_path_buf())
    //     .and_modify(|e| *e = 0) //temp
    //     .or_insert(0);
    //
    // if let Some(parent) = parent_dir {
    //     millerColumnsMap
    //         .entry(parent.to_path_buf())
    //         .and_modify(|e| *e = 0) //temp
    //         .or_insert(0);
    // }

    // let child_dir: Option
    Ok([parent_dir_files, selected_dir_files, child_dir_files])
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
