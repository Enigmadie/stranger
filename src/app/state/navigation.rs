use std::io;

use crate::app::{
    model::{
        file::{build_full_path, get_current_file},
        miller::positions::{get_position, update_dir_position, update_parent_position},
    },
    state::{FileManager, Mode, State},
};

fn update_visual_selection(state: &mut State<'_>, anchor: usize, cursor: usize) {
    state
        .marked
        .retain(|path| path.parent() != Some(state.current_dir.as_path()));
    if state.files[1].is_empty() {
        return;
    }
    let anchor = anchor.min(state.files[1].len() - 1);
    let cursor = cursor.min(state.files[1].len() - 1);
    state.mode = Mode::Visual { anchor };
    let start = anchor.min(cursor);
    let end = anchor.max(cursor);
    state.marked.extend(
        state.files[1][start..=end]
            .iter()
            .map(|file| state.current_dir.join(&file.name)),
    );
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
            let visual_anchor = match self.mode {
                Mode::Visual { anchor } => Some(anchor),
                _ => None,
            };
            self.reset_state(new_position_id)?;
            update_dir_position(&mut self.positions_map, &self.current_dir, new_position_id);
            if let Some(anchor) = visual_anchor {
                update_visual_selection(self, anchor, new_position_id);
            }
        }
        Ok(())
    }

    fn navigate_down(&mut self, step: usize) -> io::Result<()> {
        let position_id = get_position(&self.positions_map, &self.current_dir);
        let last_index = self.files[1].len().saturating_sub(1);

        if !self.files[1].is_empty() {
            let new_position_id = (position_id + step).min(last_index);
            let visual_anchor = match self.mode {
                Mode::Visual { anchor } => Some(anchor),
                _ => None,
            };

            self.reset_state(new_position_id)?;
            update_dir_position(&mut self.positions_map, &self.current_dir, new_position_id);
            if let Some(anchor) = visual_anchor {
                update_visual_selection(self, anchor, new_position_id);
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

    #[test]
    fn visual_selection_keeps_an_already_marked_anchor() {
        let temp = tempdir().unwrap();
        fs::write(temp.path().join("a"), "").unwrap();
        fs::write(temp.path().join("b"), "").unwrap();
        let mut state = create_test_state_at(temp.path()).unwrap();
        state.marked.push(temp.path().join("a"));

        state.enter_visual_mode();
        state.navigate_down(1).unwrap();

        assert_eq!(
            state.marked,
            vec![temp.path().join("a"), temp.path().join("b")]
        );
    }

    #[test]
    fn visual_selection_is_an_inclusive_range_across_the_anchor() {
        let temp = tempdir().unwrap();
        for name in ["a", "b", "c", "d"] {
            fs::write(temp.path().join(name), "").unwrap();
        }
        let mut state = create_test_state_at(temp.path()).unwrap();
        state.navigate_down(2).unwrap();
        state.enter_visual_mode();

        state.navigate_up(2).unwrap();
        assert_eq!(
            state.marked,
            vec![
                temp.path().join("a"),
                temp.path().join("b"),
                temp.path().join("c")
            ]
        );

        state.navigate_down(3).unwrap();
        assert_eq!(
            state.marked,
            vec![temp.path().join("c"), temp.path().join("d")]
        );
    }
}
