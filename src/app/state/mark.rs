use crate::app::{
    model::file::{build_full_path, get_current_file},
    state::{Navigation, State},
};

pub trait Mark {
    fn mark_item(&mut self);
    fn mark_and_down(&mut self);
    fn clear_marks(&mut self);
}

impl<'a> Mark for State<'a> {
    fn mark_item(&mut self) {
        let current_file = get_current_file(&self.positions_map, &self.current_dir, &self.files[1]);
        if let Some(file) = current_file {
            let path = build_full_path(&self.current_dir, file);
            if !self.marked.contains(&path) {
                self.marked.push(path);
            } else {
                self.marked.retain(|marked| marked != &path);
            }
        }
    }

    fn mark_and_down(&mut self) {
        let current_file = get_current_file(&self.positions_map, &self.current_dir, &self.files[1]);
        if let Some(file) = current_file {
            let path = build_full_path(&self.current_dir, file);
            if !self.marked.contains(&path) {
                self.marked.push(path);
            } else {
                self.marked.retain(|marked| marked != &path);
            }
        }
        let _ = self.navigate_down(1);
    }

    fn clear_marks(&mut self) {
        self.marked.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{state::Mode, test_utils::create_test_state};
    use std::path::PathBuf;

    #[test]
    fn mark_item_adds_file() {
        let mut state = create_test_state();
        state.mark_item();
        assert_eq!(state.marked, vec![state.current_dir.join("file1")]);
    }

    #[test]
    fn mark_item_removes_file() {
        let mut state = create_test_state();
        state.mark_item();
        let initial_length = state.marked.len();
        state.mark_item();
        assert_eq!(state.marked.len(), initial_length - 1);
    }

    #[test]
    fn marks_and_moves_down() {
        let mut state = create_test_state();
        let initial_length = state.marked.len();
        state.mark_and_down();
        assert_eq!(state.marked.len(), initial_length + 1);
        assert_eq!(state.mode, Mode::Normal);
    }

    #[test]
    fn marks_are_bound_to_their_directory() {
        let mut state = create_test_state();
        state.mark_item();
        state.current_dir = PathBuf::from("/another/directory");
        state.positions_map.insert(state.current_dir.clone(), 0);
        state.mark_item();

        assert_eq!(
            state.marked,
            vec![
                PathBuf::from("/src/ui/tests/file1"),
                PathBuf::from("/another/directory/file1")
            ]
        );
    }
}
