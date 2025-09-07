use crate::app::{
    state::State,
    ui::modal::{hint_bar::HintBarMode, ModalKind},
};

pub trait HintBar {
    fn open_hint_bar(&mut self, mode: HintBarMode);
    fn hide_hint_bar(&mut self);
}

impl<'a> HintBar for State<'a> {
    fn open_hint_bar(&mut self, mode: HintBarMode) {
        self.modal_type = ModalKind::HintBar { mode }
    }

    fn hide_hint_bar(&mut self) {
        self.modal_type = ModalKind::Disabled;
    }
}
