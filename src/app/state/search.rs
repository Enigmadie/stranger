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
    fn next_match(&mut self, direction: String) -> io::Result<()>;
    fn exit_search_mode(&mut self) -> io::Result<()>;
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
        self.next_match("next".to_string())
    }

    fn next_match(&mut self, direction: String) -> io::Result<()> {
        if let Some(pattern) = &self.search_pattern {
            let current_position = get_position(&self.positions_map, &self.current_dir);
            let files = &self.files[1];
            if files.is_empty() || pattern.is_empty() {
                return Ok(());
            }

            let start_index = match direction.as_ref() {
                "next" => (current_position + 1) % files.len(),
                "prev" => (current_position + files.len() - 1) % files.len(),
                _ => (current_position + 1) % files.len(),
            };

            let mut found_index = None;

            for i in 0..files.len() {
                let index = (start_index + i) % files.len();
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
