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
    fn commit_new_bookmark(&mut self, alias: String) -> io::Result<()>;
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
                self.reset_state(0)?;
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
            self.reset_state(0)?;
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

    fn commit_new_bookmark(&mut self, alias: String) -> io::Result<()> {
        if let Some(current_file) =
            get_current_file(&self.positions_map, &self.current_dir, &self.files[1])
        {
            let full_path = build_full_path(&self.current_dir, current_file);
            let mut config = self.config.clone();
            config.bookmarks.insert(alias, full_path);
            save_config(&config)?;
            self.config = config;

            self.notification = Some(Notification::Info {
                msg: Lang::en("bookmark_added").into(),
            });
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                Lang::en("items_not_found"),
            ))
        }
    }

    fn delete_from_bookmarks(&mut self) {
        if let Mode::Bookmarks { position_id } = self.mode {
            let mut config = self.config.clone();
            config.bookmarks.swap_remove_index(position_id);
            match save_config(&config) {
                Ok(()) => {
                    self.config = config;
                    self.notification = Notification::Info {
                        msg: Lang::en("bookmark_deleted").into(),
                    }
                    .into();
                }
                Err(error) => {
                    self.notification = Notification::Error {
                        msg: error.to_string().into(),
                    }
                    .into();
                }
            }
        }
    }

    fn open_dir_from_bookmark(&mut self) -> io::Result<()> {
        if let Mode::Bookmarks { position_id } = self.mode {
            if let Some((_, value)) = self.config.bookmarks.get_index(position_id) {
                match () {
                    _ if value.is_dir() => {
                        let target_dir = value.clone();
                        let millers_id = get_position(&self.positions_map, &target_dir);
                        self.reset_state_to(target_dir, millers_id)?;
                        self.mode = Mode::Normal;
                    }
                    _ if value.is_file() => {
                        let file = value.clone();
                        self.execute_file(&file)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{
        model::miller::positions::{get_position, update_dir_position},
        test_utils::create_test_state_at,
    };
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn opening_bookmark_uses_the_target_directory_position() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("target");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("a"), "").unwrap();
        fs::write(target.join("b"), "").unwrap();
        let mut state = create_test_state_at(temp.path()).unwrap();
        update_dir_position(&mut state.positions_map, &target, 1);
        state
            .config
            .bookmarks
            .insert("target".into(), target.clone());
        state.mode = Mode::Bookmarks { position_id: 0 };

        state.open_dir_from_bookmark().unwrap();

        assert_eq!(state.current_dir, target);
        assert_eq!(get_position(&state.positions_map, &state.current_dir), 1);
        assert_eq!(state.files[1][1].name, "b");
    }
}
