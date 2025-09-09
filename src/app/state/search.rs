use crate::app::{
    state::{Mode, State},
    ui::modal::ModalKind,
};

pub trait Search {
    fn search(&mut self);
    fn commit_search(&mut self);
}

impl<'a> Search for State<'a> {
    fn search(&mut self) {
        self.mode = Mode::Insert;
        self.modal_type = ModalKind::BottomLine;
    }

    fn commit_search(&mut self) {
        let query = self.input.lines().join("").to_lowercase();

        let curent_dir = self.files[1]
            .clone()
            .into_iter()
            .filter(|f| f.name.to_lowercase().starts_with(&query))
            .collect();

        self.files[1] = curent_dir;
        self.setup_default_input();
    }
}
