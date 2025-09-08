use std::io;

use crate::app::{
    model::{
        file::{build_full_path, get_current_file},
        miller::positions::get_position,
        notification::Notification,
    },
    state::{FileManager, HintBar, Mode, State},
    ui::modal::ModalKind,
    utils::{config_parser::save_config, i18n::Lang},
};

pub trait Bookmarks {
    fn bookmarks_nagivate_down(&mut self) -> io::Result<()>;
    fn bookmarks_nagivate_up(&mut self) -> io::Result<()>;
    fn enter_bookmarks_mode(&mut self);
    fn add_to_bookmarks(&mut self);
    fn commit_new_bookmark(&mut self, alias: String);
    fn delete_from_bookmarks(&mut self);
    fn open_dir_from_bookmark(&mut self) -> io::Result<()>;
}

impl<'a> Bookmarks for State<'a> {
    fn bookmarks_nagivate_down(&mut self) -> io::Result<()> {
        if let Mode::Bookmarks { position_id } = self.mode {
            let incremented_position = position_id.saturating_add(1);
            if incremented_position < self.config.bookmarks.len() {
                let new_position_id = incremented_position;
                self.mode = Mode::Bookmarks {
                    position_id: new_position_id,
                };
                let _ = self.reset_state(0);
            }
        }
        Ok(())
    }

    fn bookmarks_nagivate_up(&mut self) -> io::Result<()> {
        if let Mode::Bookmarks { position_id } = self.mode {
            let new_position_id = position_id.saturating_sub(1);
            self.mode = Mode::Bookmarks {
                position_id: new_position_id,
            };
            let _ = self.reset_state(0);
        }
        Ok(())
    }

    fn enter_bookmarks_mode(&mut self) {
        self.mode = Mode::Bookmarks { position_id: 0 };
        self.hide_hint_bar();
        self.notification = Notification::Info {
            msg: Lang::en("bookmarks_mode").into(),
        }
        .into();
    }

    fn add_to_bookmarks(&mut self) {
        self.enter_insert_mode();
        self.modal_type = ModalKind::UnderLine {
            action: crate::app::ui::modal::UnderLineModalAction::Bookmarks,
        };
    }

    fn commit_new_bookmark(&mut self, alias: String) {
        if let Some(current_file) =
            get_current_file(&self.positions_map, &self.current_dir, &self.files[1])
        {
            let full_path = build_full_path(&self.current_dir, current_file);
            self.config.bookmarks.insert(alias, full_path);

            let _ = save_config(&self.config);

            self.notification = Notification::Info {
                msg: Lang::en("bookmark_added").into(),
            }
            .into()
        }
    }

    fn delete_from_bookmarks(&mut self) {
        if let Mode::Bookmarks { position_id } = self.mode {
            self.config.bookmarks.swap_remove_index(position_id);
            let _ = save_config(&self.config);

            self.notification = Notification::Info {
                msg: Lang::en("bookmark_deleted").into(),
            }
            .into()
        }
    }

    fn open_dir_from_bookmark(&mut self) -> io::Result<()> {
        if let Mode::Bookmarks { position_id } = self.mode {
            if let Some((_, value)) = self.config.bookmarks.get_index(position_id) {
                let millers_id = get_position(&self.positions_map, &self.current_dir);
                match () {
                    _ if value.is_dir() => {
                        self.current_dir = value.clone();
                        let _ = self.reset_state(millers_id);
                        self.mode = Mode::Normal;
                    }
                    _ if value.is_file() => {
                        self.execute_file(value.clone());
                    }
                    _ => {
                        self.notification = Notification::Error {
                            msg: Lang::en("bookmark_invalid").into(),
                        }
                        .into();
                    }
                }
            }
        }
        Ok(())
    }
}
