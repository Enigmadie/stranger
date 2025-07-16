use std::{collections::HashMap, path::PathBuf};
use tui_textarea::TextArea;

use crate::app::{
    model::file_entry::{FileEntry, FileVariant},
    state::{Mode, State},
    ui::modal::ModalKind,
};

pub fn create_test_state() -> State<'static> {
    let mut positions_map: HashMap<PathBuf, usize> = HashMap::new();
    let current_dir = PathBuf::from("/src/ui/tests");

    positions_map.insert(current_dir.clone(), 0);

    State {
        current_dir,
        files: [
            vec![],
            vec![FileEntry {
                name: "file1".into(),
                variant: FileVariant::File,
            }],
            vec![],
        ],
        dirs: [
            Some(PathBuf::from("src/ui/1")),
            Some(PathBuf::from("src/ui/2")),
            Some(PathBuf::from("src/ui/3")),
        ],
        mode: Mode::Normal,
        show_popup: false,
        modal_type: ModalKind::UnderLine,
        positions_map,
        input: TextArea::default(),
        err_msg: None,
    }
}
