use crate::app::{
    config::constants::model::{NUM_COLUMNS, ZERO_POSITION},
    model::miller::entries::FileEntry,
};
use std::{collections::HashMap, path::PathBuf};

pub fn parse_path_positions(
    current_dir: &PathBuf,
    column_files: &[Vec<FileEntry>; NUM_COLUMNS],
) -> HashMap<PathBuf, usize> {
    let mut positions = HashMap::new();
    positions.insert(current_dir.to_path_buf(), ZERO_POSITION);

    update_parent_position(&mut positions, current_dir, column_files);
    positions
}

pub fn update_parent_position(
    positions: &mut HashMap<PathBuf, usize>,
    current_dir: &PathBuf,
    column_files: &[Vec<FileEntry>; NUM_COLUMNS],
) {
    if let Some(parent_name_os) = &current_dir.file_name() {
        let parent_name = parent_name_os.to_string_lossy();
        if let Some(parent_position) = column_files[0].iter().position(|f| f.name == *parent_name) {
            if let Some(parent_dir) = current_dir.parent() {
                positions.insert(parent_dir.to_path_buf(), parent_position);
            }
        }
    }
}

pub fn get_position(positions: &HashMap<PathBuf, usize>, dir: &PathBuf) -> usize {
    let position_id = positions.get(dir).copied().unwrap_or(ZERO_POSITION);
    position_id
}

pub fn update_dir_position(
    positions: &mut HashMap<PathBuf, usize>,
    dir: &PathBuf,
    new_position_id: usize,
) {
    positions.insert(dir.clone(), new_position_id);
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::app::test_utils::create_test_state;

    #[test]
    fn init_positions() {
        let state = create_test_state();
        let positions = parse_path_positions(&state.current_dir, &state.files);

        assert_eq!(positions.len(), 1);
        assert_eq!(positions.get(&state.current_dir), Some(&ZERO_POSITION));
    }

    #[test]
    fn get_position_id() {
        let path = PathBuf::from("/src/ui/tests");
        let mut positions: HashMap<PathBuf, usize> = HashMap::new();
        positions.insert(path.clone(), 5);

        let position_id = get_position(&positions, &path);

        assert_eq!(position_id, 5);

        let missing_path = PathBuf::from("/src/ui/lib");
        let position_id = get_position(&positions, &missing_path);

        assert_eq!(position_id, ZERO_POSITION);
    }

    #[test]
    fn update_positions() {
        let state = create_test_state();
        let mut positions = parse_path_positions(&state.current_dir, &state.files);

        update_dir_position(&mut positions, &state.current_dir, 5);

        assert_eq!(positions.get(&state.current_dir), Some(&5));
    }
}
