use std::{io, path::PathBuf};

use crate::app::{
    model::{
        file::{build_full_path, get_current_file},
        miller::positions::{get_position, update_dir_position, update_parent_position},
    },
    state::{FileManager, Mode, State},
};

fn update_visual_selection(state: &mut State<'_>, previous: PathBuf, target: PathBuf) {
    if state.marked.contains(&target) {
        state.marked.retain(|path| path != &previous);
    } else {
        state.marked.push(target);
    }
    state.mode = Mode::Visual { init: false };
}

pub trait Navigation {
    fn navigate_to_child(&mut self) -> io::Result<()>;
    fn navigate_to_parent(&mut self) -> io::Result<()>;
    fn navigate_up(&mut self, step: usize) -> io::Result<()>;
    fn navigate_down(&mut self, step: usize) -> io::Result<()>;
    fn navigate_to_child_or_exec(&mut self) -> io::Result<()>;
}

impl<'a> Navigation for State<'a> {
    fn navigate_to_parent(&mut self) -> io::Result<()> {
        if let Some(parent) = &self.dirs[0].dir_name {
            let target_dir = parent.to_path_buf();
            let position_id = get_position(&self.positions_map, &target_dir);
            self.reset_state_to(target_dir, position_id)?;
            update_parent_position(&mut self.positions_map, &self.current_dir, &self.files);
        }
        Ok(())
    }

    fn navigate_to_child(&mut self) -> io::Result<()> {
        if let Some(child) = &self.dirs[2].dir_name {
            let target_dir = child.to_path_buf();
            let position_id = get_position(&self.positions_map, &target_dir);
            self.reset_state_to(target_dir, position_id)?;
        }
        Ok(())
    }

    fn navigate_up(&mut self, step: usize) -> io::Result<()> {
        let position_id = get_position(&self.positions_map, &self.current_dir);
        if position_id > 0 {
            let new_position_id = position_id.saturating_sub(step);
            let previous = self.files[1]
                .get(position_id)
                .map(|file| build_full_path(&self.current_dir, file));
            let target = self.files[1]
                .get(new_position_id)
                .map(|file| build_full_path(&self.current_dir, file));
            self.reset_state(new_position_id)?;
            update_dir_position(&mut self.positions_map, &self.current_dir, new_position_id);
            if matches!(self.mode, Mode::Visual { .. }) {
                if let (Some(previous), Some(target)) = (previous, target) {
                    update_visual_selection(self, previous, target);
                }
            }
        }
        Ok(())
    }

    fn navigate_down(&mut self, step: usize) -> io::Result<()> {
        let position_id = get_position(&self.positions_map, &self.current_dir);
        let last_index = self.files[1].len().saturating_sub(1);

        if !self.files[1].is_empty() {
            let new_position_id = (position_id + step).min(last_index);
            let previous = self.files[1]
                .get(position_id)
                .map(|file| build_full_path(&self.current_dir, file));
            let target = self.files[1]
                .get(new_position_id)
                .map(|file| build_full_path(&self.current_dir, file));

            self.reset_state(new_position_id)?;
            update_dir_position(&mut self.positions_map, &self.current_dir, new_position_id);
            if matches!(self.mode, Mode::Visual { .. }) {
                if let (Some(previous), Some(target)) = (previous, target) {
                    update_visual_selection(self, previous, target);
                }
            }
        }

        Ok(())
    }

    fn navigate_to_child_or_exec(&mut self) -> io::Result<()> {
        let current_file = get_current_file(&self.positions_map, &self.current_dir, &self.files[1]);
        if let Some(file) = current_file {
            let file_path = build_full_path(&self.current_dir, file);
            if file.variant.is_regular_file() {
                self.execute_file(&file_path)?;
            } else {
                self.navigate_to_child()?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::test_utils::create_test_state_at;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_navigate_to_parent() {
        let temp = tempdir().unwrap();
        let child = temp.path().join("child");
        fs::create_dir(&child).unwrap();
        let mut state = create_test_state_at(&child).unwrap();

        assert!(state.navigate_to_parent().is_ok());
        assert_eq!(state.current_dir, temp.path());
    }

    #[test]
    fn test_navigate_to_child() {
        let temp = tempdir().unwrap();
        let child = temp.path().join("child");
        fs::create_dir(&child).unwrap();
        let mut state = create_test_state_at(temp.path()).unwrap();

        assert!(state.navigate_to_child().is_ok());
        assert_eq!(state.current_dir, child);
    }

    #[test]
    fn test_navigate_up() {
        let temp = tempdir().unwrap();
        fs::write(temp.path().join("a"), "").unwrap();
        fs::write(temp.path().join("b"), "").unwrap();
        let mut state = create_test_state_at(temp.path()).unwrap();
        update_dir_position(&mut state.positions_map, &state.current_dir, 1);
        state.reset_state(1).unwrap();

        assert!(state.navigate_up(1).is_ok());
        let new_position = get_position(&state.positions_map, &state.current_dir);
        assert_eq!(new_position, 0);
    }

    #[test]
    fn test_navigate_up_at_zero() {
        let temp = tempdir().unwrap();
        fs::write(temp.path().join("a"), "").unwrap();
        let mut state = create_test_state_at(temp.path()).unwrap();

        assert!(state.navigate_up(1).is_ok());
        let new_position = get_position(&state.positions_map, &state.current_dir);
        assert_eq!(new_position, 0);
    }

    #[test]
    fn test_navigate_down() {
        let temp = tempdir().unwrap();
        fs::write(temp.path().join("a"), "").unwrap();
        fs::write(temp.path().join("b"), "").unwrap();
        let mut state = create_test_state_at(temp.path()).unwrap();

        assert!(state.navigate_down(1).is_ok());
        let new_position = get_position(&state.positions_map, &state.current_dir);
        assert_eq!(new_position, 1);
    }

    #[test]
    fn failed_navigation_keeps_the_current_snapshot() {
        let temp = tempdir().unwrap();
        fs::write(temp.path().join("file"), "").unwrap();
        let mut state = create_test_state_at(temp.path()).unwrap();
        let original_dir = state.current_dir.clone();
        let original_files = state.files[1].clone();
        state.dirs[2].dir_name = Some(temp.path().join("missing"));

        assert!(state.navigate_to_child().is_err());
        assert_eq!(state.current_dir, original_dir);
        assert_eq!(state.files[1], original_files);
    }

    #[test]
    fn visual_selection_contracts_when_navigating_back() {
        let temp = tempdir().unwrap();
        fs::write(temp.path().join("a"), "").unwrap();
        fs::write(temp.path().join("b"), "").unwrap();
        let mut state = create_test_state_at(temp.path()).unwrap();
        state.enter_visual_mode();

        state.navigate_down(1).unwrap();
        assert_eq!(state.marked.len(), 2);

        state.navigate_up(1).unwrap();
        assert_eq!(state.marked, vec![temp.path().join("a")]);
    }
}
