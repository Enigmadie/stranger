use std::{collections::HashMap, path::PathBuf};

pub const ZERO_POSITION: usize = 0;

pub fn parse_path_positions(current_dir: &PathBuf) -> HashMap<PathBuf, usize> {
    let mut positions = HashMap::new();
    positions.insert(current_dir.clone(), ZERO_POSITION);
    positions
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

    #[test]
    fn init_positions() {
        let path = PathBuf::from("/src/ui/tests");
        let positions = parse_path_positions(&path);

        assert_eq!(positions.len(), 1);
        assert_eq!(positions.get(&path), Some(&ZERO_POSITION));
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
        let path = PathBuf::from("/src/ui/tests");
        let mut positions = parse_path_positions(&path);

        update_dir_position(&mut positions, &path, 5);

        assert_eq!(positions.get(&path), Some(&5));
    }
}
