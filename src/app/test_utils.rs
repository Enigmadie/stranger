use crate::app::ui::file_preview::PreviewCache;
use std::{collections::HashMap, path::PathBuf};
use tui_textarea::TextArea;

use crate::app::{
    model::miller::{
        columns::MillerColumns,
        entries::{DirEntry, FileEntry, FileVariant},
        positions::parse_path_positions,
    },
    state::{Mode, State},
    ui::modal::ModalKind,
    utils::config_parser::default_config::Config,
};

pub fn create_test_state_at(path: &std::path::Path) -> std::io::Result<State<'static>> {
    let current_dir = path.to_path_buf();
    let columns = MillerColumns::build_columns(&current_dir, 0, None, false)?;
    let positions_map = parse_path_positions(&current_dir, &columns.files);

    Ok(State {
        current_dir,
        files: columns.files,
        dirs: columns.dirs,
        mode: Mode::Normal,
        modal_type: ModalKind::Disabled,
        positions_map,
        input: TextArea::default(),
        config: Config::default(),
        from_external_app: false,
        clipboard: None,
        notification: None,
        marked: vec![],
        search_pattern: None,
        show_hidden_files: false,
        preview: Vec::new(),
        preview_cache: PreviewCache::default(),
    })
}

pub fn create_test_state() -> State<'static> {
    let mut positions_map: HashMap<PathBuf, usize> = HashMap::new();
    let current_dir = PathBuf::from("/src/ui/tests");

    positions_map.insert(current_dir.clone(), 0);

    State {
        current_dir,
        files: [
            vec![],
            vec![
                FileEntry {
                    name: "file1".into(),
                    display_name: "file1".into(),
                    variant: FileVariant::File {
                        size: Some(10),
                        permissions: Some("rwxr-xr-x".into()),
                        last_modified: Some("2023-10-01 12:00".into()),
                        is_matched: false,
                    },
                },
                FileEntry {
                    name: "file2".into(),
                    display_name: "file2".into(),
                    variant: FileVariant::File {
                        size: Some(10),
                        permissions: Some("rwxr-xr-x".into()),
                        last_modified: Some("2023-10-01 12:00".into()),
                        is_matched: false,
                    },
                },
                FileEntry {
                    name: "file3".into(),
                    display_name: "file3".into(),
                    variant: FileVariant::File {
                        size: Some(10),
                        permissions: Some("rwxr-xr-x".into()),
                        last_modified: Some("2023-10-01 12:00".into()),
                        is_matched: false,
                    },
                },
            ],
            vec![],
        ],
        dirs: [
            DirEntry {
                dir_name: Some(PathBuf::from("src/ui/1")),
                with_meta: false,
            },
            DirEntry {
                dir_name: Some(PathBuf::from("src/ui/1")),
                with_meta: false,
            },
            DirEntry {
                dir_name: Some(PathBuf::from("src/ui/1")),
                with_meta: false,
            },
        ],
        mode: Mode::Normal,
        modal_type: ModalKind::Disabled,
        positions_map,
        input: TextArea::default(),
        config: Config::default(),
        from_external_app: false,
        clipboard: None,
        notification: None,
        marked: vec![],
        search_pattern: None,
        show_hidden_files: false,
        preview: Vec::new(),
        preview_cache: PreviewCache::default(),
    }
}
