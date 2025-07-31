use crate::app::{
    model::{
        file::{build_full_path, get_current_file},
        miller::{columns::MillerColumns, positions::get_position},
        notification::Notification,
    },
    state::State,
    ui::modal::{ModalKind, UnderLineModalAction},
    utils::fs::{create_dir, create_file, rename_file},
};

pub trait Modal {
    fn add(&mut self);
    fn rename(&mut self);
    fn commit(&mut self);
}

impl<'a> Modal for State<'a> {
    fn add(&mut self) {
        self.start_editing();
        self.modal_type = ModalKind::UnderLine {
            action: UnderLineModalAction::Add,
        };
    }

    fn rename(&mut self) {
        let includes_files = MillerColumns::check_is_current_dir_is_not_empty(&self.files[1]);
        if includes_files {
            self.start_editing();
            self.modal_type = ModalKind::UnderLine {
                action: UnderLineModalAction::Edit,
            };
        }
    }

    fn commit(&mut self) {
        let input_value = self.input.lines().join("");

        match &self.modal_type {
            ModalKind::UnderLine { action } => match action {
                UnderLineModalAction::Add => {
                    let is_dir = self.input.lines().last().is_some_and(|e| e.ends_with('/'));

                    if is_dir {
                        let _ = create_dir(input_value);
                    } else {
                        let _ = create_file(input_value);
                    }
                    let _ = self.reset_state(0);
                }
                UnderLineModalAction::Edit => {
                    let current_file =
                        get_current_file(&self.positions_map, &self.current_dir, &self.files[1]);
                    if let Some(file) = current_file {
                        let full_path = build_full_path(&self.current_dir, file);
                        let _ = rename_file(&full_path, input_value);
                        let positiond_id = get_position(&self.positions_map, &self.current_dir);
                        let _ = self.reset_state(positiond_id);
                    } else {
                        self.notification = Some(Notification::Error {
                            msg: format!("Failed to update file: {}", self.current_dir.display())
                                .into(),
                        });
                    }
                }
            },
        };

        self.enter_normal_mode();
        self.setup_default_input();
    }
}
