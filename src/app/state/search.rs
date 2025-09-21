use crate::app::{
    model::miller::positions::get_position,
    state::{Mode, State},
    ui::modal::ModalKind,
};

pub trait Search {
    fn search(&mut self);
    fn commit_search(&mut self);
    fn next_search(&mut self);
}

impl<'a> Search for State<'a> {
    fn search(&mut self) {
        self.mode = Mode::Insert;
        self.modal_type = ModalKind::BottomLine;
    }

    fn commit_search(&mut self) {
        let query = self.input.lines().join("").to_lowercase();

        self.search_pattern = Some(query);
        self.setup_default_input();
        let positiond_id = get_position(&self.positions_map, &self.current_dir);
        let _ = self.reset_state(positiond_id);
    }

    fn next_search(&mut self) {
        let query = self.input.lines().join("").to_lowercase();

        self.search_pattern = Some(query);
        self.setup_default_input();
        let positiond_id = get_position(&self.positions_map, &self.current_dir);
        let _ = self.reset_state_except_notifications(positiond_id);
    }
}
