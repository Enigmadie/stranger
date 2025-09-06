use crate::app::{
    state::State,
    ui::modal::{HintBarMode, ModalKind},
};

pub trait HintBar {
    fn commit_hint_bar(&mut self);
    fn enter_bookmark_hint_bar(&mut self);
}

impl<'a> HintBar for State<'a> {
    fn enter_bookmark_hint_bar(&mut self) {
        self.modal_type = ModalKind::HintBar {
            mode: HintBarMode::Bookmarks,
        }
    }

    fn commit_hint_bar(&mut self) {
        if let ModalKind::HintBar { mode } = &self.modal_type {
            match mode {
                HintBarMode::Bookmarks => {
                    // Commit bookmark logic here
                }
            }
        }
    }
}
