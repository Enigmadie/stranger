use std::io;

use crate::app::{
    model::miller::positions::{get_position, update_dir_position},
    state::{Mode, State},
    ui::modal::ModalKind,
    utils::i18n::Lang,
};

pub trait Search {
    fn search(&mut self);
    fn commit_search(&mut self) -> io::Result<()>;
    fn next_match(&mut self, direction: SearchDirection) -> io::Result<()>;
    fn exit_search_mode(&mut self) -> io::Result<()>;
}

#[derive(Clone, Copy)]
pub enum SearchDirection {
    Forward,
    Backward,
}

impl<'a> Search for State<'a> {
    fn search(&mut self) {
        self.mode = Mode::Insert;
        self.modal_type = ModalKind::BottomLine;
    }

    fn commit_search(&mut self) -> io::Result<()> {
        let query = self.input.lines().join("").to_lowercase();

        self.search_pattern = Some(query);
        let positiond_id = get_position(&self.positions_map, &self.current_dir);
        self.setup_default_input();
        self.mode = Mode::Search;
        self.reset_state(positiond_id)?;
        self.next_match(SearchDirection::Forward)
    }

    fn next_match(&mut self, direction: SearchDirection) -> io::Result<()> {
        if let Some(pattern) = &self.search_pattern {
            let current_position = get_position(&self.positions_map, &self.current_dir);
            let files = &self.files[1];
            if files.is_empty() || pattern.is_empty() {
                return Ok(());
            }

            let mut found_index = None;

            for distance in 1..=files.len() {
                let index = match direction {
                    SearchDirection::Forward => (current_position + distance) % files.len(),
                    SearchDirection::Backward => {
                        (current_position + files.len() - (distance % files.len())) % files.len()
                    }
                };
                if files[index].variant.is_matched() {
                    found_index = Some(index);
                    break;
                }
            }

            if let Some(new_position) = found_index {
                self.reset_state(new_position)?;
                update_dir_position(&mut self.positions_map, &self.current_dir, new_position);
            } else {
                self.notification = Some(crate::app::state::Notification::Info {
                    msg: Lang::en_fmt("no_matches", &[pattern]).into(),
                });
            }
        }
        Ok(())
    }

    fn exit_search_mode(&mut self) -> io::Result<()> {
        self.mode = Mode::Normal;
        self.search_pattern = None;
        let positiond_id = get_position(&self.positions_map, &self.current_dir);
        self.reset_state(positiond_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{
        model::miller::{entries::FileVariant, positions::get_position},
        test_utils::create_test_state_at,
    };
    use std::fs;
    use tempfile::tempdir;

    fn set_matched(file: &mut crate::app::model::miller::entries::FileEntry) {
        match &mut file.variant {
            FileVariant::File { is_matched, .. }
            | FileVariant::Directory { is_matched, .. }
            | FileVariant::Symlink { is_matched }
            | FileVariant::Special { is_matched } => *is_matched = true,
        }
    }

    #[test]
    fn search_traverses_in_the_requested_direction() {
        let temp = tempdir().unwrap();
        for name in ["a", "b", "c"] {
            fs::write(temp.path().join(name), "").unwrap();
        }
        let mut state = create_test_state_at(temp.path()).unwrap();
        state.search_pattern = Some("match".into());
        set_matched(&mut state.files[1][0]);
        set_matched(&mut state.files[1][2]);
        state.reset_state(1).unwrap();
        set_matched(&mut state.files[1][0]);
        set_matched(&mut state.files[1][2]);

        state.next_match(SearchDirection::Backward).unwrap();
        assert_eq!(get_position(&state.positions_map, &state.current_dir), 0);

        state.reset_state(1).unwrap();
        set_matched(&mut state.files[1][0]);
        set_matched(&mut state.files[1][2]);
        state.next_match(SearchDirection::Forward).unwrap();
        assert_eq!(get_position(&state.positions_map, &state.current_dir), 2);
    }
}
