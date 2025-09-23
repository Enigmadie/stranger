use crate::app::{
    model::file::get_current_file,
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
            let found_file = self.marked.iter().any(|f| f.name == file.name);
            if !found_file {
                self.marked.push(file.clone());
            } else {
                self.marked.retain(|e| e.name != file.name);
            }
        }
    }

    fn mark_and_down(&mut self) {
        let current_file = get_current_file(&self.positions_map, &self.current_dir, &self.files[1]);
        if let Some(file) = current_file {
            let found_file = &self.marked.iter().any(|f| f.name == file.name);
            if !found_file {
                self.marked.push(file.clone());
            } else {
                self.marked.retain(|e| e.name != file.name);
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

    #[test]
    fn mark_item_adds_file() {
        let mut state = create_test_state();
        let initial_length = state.marked.len();
        state.mark_item();
        assert_eq!(state.marked.len(), initial_length + 1);
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
}
