use std::io;

use crate::app::{
    model::miller::positions::{get_position, update_dir_position},
    state::{Mode, State},
};

pub trait Navigation {
    fn navigate_to_child(&mut self) -> io::Result<()>;
    fn navigate_to_parent(&mut self) -> io::Result<()>;
    fn navigate_up(&mut self, step: usize) -> io::Result<()>;
    fn navigate_down(&mut self, step: usize) -> io::Result<()>;
}

impl<'a> Navigation for State<'a> {
    fn navigate_to_parent(&mut self) -> io::Result<()> {
        if let Some(parent) = &self.dirs[0].dir_name {
            self.current_dir = parent.to_path_buf();
            let position_id = get_position(&self.positions_map, &self.current_dir);
            let _ = self.reset_state(position_id);
        }
        Ok(())
    }

    fn navigate_to_child(&mut self) -> io::Result<()> {
        if let Some(child) = &self.dirs[2].dir_name {
            self.current_dir = child.to_path_buf();
            let position_id = get_position(&self.positions_map, &self.current_dir);
            let _ = self.reset_state(position_id);
        }
        Ok(())
    }

    fn navigate_up(&mut self, step: usize) -> io::Result<()> {
        let position_id = get_position(&self.positions_map, &self.current_dir);
        if position_id > 0 {
            let new_position_id = position_id.saturating_sub(step);

            if let Mode::Visual { init } = self.mode {
                if init {
                    // first time run after in visual mode
                    self.mode = Mode::Visual { init: false }
                } else {
                    self.mark_item();
                }
            }
            update_dir_position(&mut self.positions_map, &self.current_dir, new_position_id);
            let _ = self.reset_state(new_position_id);
        }
        Ok(())
    }

    fn navigate_down(&mut self, step: usize) -> io::Result<()> {
        let position_id = get_position(&self.positions_map, &self.current_dir);
        let last_index = self.files[1].len().saturating_sub(1);

        if !self.files[1].is_empty() {
            let new_position_id = (position_id + step).min(last_index);

            update_dir_position(&mut self.positions_map, &self.current_dir, new_position_id);
            if matches!(self.mode, Mode::Visual { .. }) {
                self.mode = Mode::Visual { init: false };
                self.mark_item();
            }
            let _ = self.reset_state(new_position_id);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::test_utils::create_test_state;

    #[test]
    fn test_navigate_to_parent() {
        let mut state = create_test_state();
        let initial_dir = state.current_dir.clone();
        state.dirs[0].dir_name = Some(initial_dir.parent().unwrap().to_path_buf());
        assert!(state.navigate_to_parent().is_ok());
        assert_eq!(state.current_dir, initial_dir.parent().unwrap());
    }

    #[test]
    fn test_navigate_to_child() {
        let mut state = create_test_state();
        let initial_dir = state.current_dir.clone();
        state.dirs[2].dir_name = Some(initial_dir.join("child"));
        assert!(state.navigate_to_child().is_ok());
        assert_eq!(state.current_dir, state.dirs[2].dir_name.clone().unwrap());
    }

    #[test]
    fn test_navigate_up() {
        let mut state = create_test_state();
        let initial_position = get_position(&state.positions_map, &state.current_dir);
        assert!(state.navigate_up().is_ok());
        let new_position = get_position(&state.positions_map, &state.current_dir);
        assert_eq!(new_position, initial_position.saturating_sub(1));
    }

    #[test]
    fn test_navigate_up_at_zero() {
        let mut state = create_test_state();
        let initial_position = 0;
        update_dir_position(
            &mut state.positions_map,
            &state.current_dir,
            initial_position,
        );
        assert!(state.navigate_up().is_ok());
        let new_position = get_position(&state.positions_map, &state.current_dir);
        assert_eq!(new_position, 0);
    }

    #[test]
    fn test_navigate_down() {
        let mut state = create_test_state();
        let initial_position = get_position(&state.positions_map, &state.current_dir);
        assert!(state.navigate_down().is_ok());
        let new_position = get_position(&state.positions_map, &state.current_dir);
        assert_eq!(new_position, initial_position.saturating_add(1));
    }
}
