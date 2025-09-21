use crate::app::{
    model::miller::{
        entries::FileVariant,
        positions::{get_position, update_dir_position},
    },
    state::{Mode, State},
    ui::modal::ModalKind,
    utils::i18n::Lang,
};

pub trait Search {
    fn search(&mut self);
    fn commit_search(&mut self);
    fn next_match(&mut self, direction: String);
    fn exit_search_mode(&mut self);
}

impl<'a> Search for State<'a> {
    fn search(&mut self) {
        self.mode = Mode::Insert;
        self.modal_type = ModalKind::BottomLine;
    }

    fn commit_search(&mut self) {
        let query = self.input.lines().join("").to_lowercase();

        self.search_pattern = Some(query);
        let positiond_id = get_position(&self.positions_map, &self.current_dir);
        self.setup_default_input();
        self.mode = Mode::Search;
        let _ = self.reset_state(positiond_id);
    }

    fn next_match(&mut self, direction: String) {
        if let Some(pattern) = &self.search_pattern {
            let current_position = get_position(&self.positions_map, &self.current_dir);
            let files = &self.files[1];
            if files.is_empty() || pattern.is_empty() {
                return;
            }

            let start_index = match direction.as_ref() {
                "next" => (current_position + 1) % files.len(),
                "prev" => (current_position + files.len() - 1) % files.len(),
                _ => (current_position + 1) % files.len(),
            };

            let mut found_index = None;

            for i in 0..files.len() {
                let index = (start_index + i) % files.len();
                if match files[index].variant {
                    FileVariant::Directory { is_matched, .. } => is_matched,
                    FileVariant::File { is_matched, .. } => is_matched,
                } {
                    found_index = Some(index);
                    break;
                }
            }

            if let Some(new_position) = found_index {
                update_dir_position(&mut self.positions_map, &self.current_dir, new_position);
                let _ = self.reset_state(new_position);
            } else {
                self.notification = Some(crate::app::state::Notification::Info {
                    msg: Lang::en_fmt("no_matches", &[pattern]).into(),
                });
            }
        }
    }

    fn exit_search_mode(&mut self) {
        self.mode = Mode::Normal;
        self.search_pattern = None;
        let positiond_id = get_position(&self.positions_map, &self.current_dir);
        let _ = self.reset_state(positiond_id);
    }
}
