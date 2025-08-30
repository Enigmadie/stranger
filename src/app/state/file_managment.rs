use std::{io, path::PathBuf};

use crate::app::{
    model::{
        clipboard::{Clipboard, ClipboardAction},
        file::{build_full_path, get_current_file},
        miller::{columns::MillerColumns, positions::get_position},
        notification::Notification,
    },
    state::{Bookmarks, State},
    ui::modal::{ModalKind, UnderLineModalAction},
    utils::{
        fs::{copy_file_path, create_dir, create_file, exec, paste_file, remove_file, rename_file},
        i18n::Lang,
    },
};

pub trait FileManager {
    fn add_file(&mut self);
    fn rename_file(&mut self);
    fn copy_files(&mut self);
    fn paste_files(&mut self) -> io::Result<()>;
    fn delete_files(&mut self);
    fn commit_changes(&mut self);
    fn execute_file(&mut self, file_name: PathBuf);
}

impl<'a> FileManager for State<'a> {
    fn add_file(&mut self) {
        self.enter_insert_mode();
        self.modal_type = ModalKind::UnderLine {
            action: UnderLineModalAction::Add,
        };
    }

    fn rename_file(&mut self) {
        let includes_files = MillerColumns::check_is_current_dir_is_not_empty(&self.files[1]);
        if includes_files {
            self.enter_insert_mode();
            self.modal_type = ModalKind::UnderLine {
                action: UnderLineModalAction::Edit,
            };
        }
    }

    fn commit_changes(&mut self) {
        let input_value = self.input.lines().join("");

        match &self.modal_type {
            ModalKind::UnderLine { action } => match action {
                UnderLineModalAction::Add => {
                    let is_dir = self.input.lines().last().is_some_and(|e| e.ends_with('/'));

                    if is_dir {
                        let _ = create_dir(input_value);
                    } else {
                        let _ = create_file(input_value, &self.current_dir);
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
                UnderLineModalAction::Bookmarks => {
                    let input_value = self.input.lines().join("");
                    self.commit_new_bookmark(input_value);
                }
            },
        };

        self.enter_normal_mode();
        self.setup_default_input();
    }

    fn copy_files(&mut self) {
        let files_to_copy = if !self.marked.is_empty() {
            self.marked.clone()
        } else {
            vec![
                get_current_file(&self.positions_map, &self.current_dir, &self.files[1])
                    .unwrap()
                    .clone(),
            ]
        };
        let copied_filepaths: Result<Vec<PathBuf>, _> = files_to_copy
            .iter()
            .map(|file| {
                let file_path = build_full_path(&self.current_dir, file);
                copy_file_path(file_path)
            })
            .collect();

        match copied_filepaths {
            Ok(value) => {
                self.notification = Notification::Success {
                    msg: Lang::en_fmt("copied", format_args!("{}", value.len())).into(),
                }
                .into();
                self.clipboard = Clipboard::File {
                    items: value,
                    action: ClipboardAction::Copy,
                }
                .into();
            }
            Err(err) => {
                self.notification = Notification::Error {
                    msg: err.to_string().into(),
                }
                .into();
            }
        }
    }

    fn delete_files(&mut self) {
        let files_to_delete = if !self.marked.is_empty() {
            self.marked.clone()
        } else {
            vec![
                get_current_file(&self.positions_map, &self.current_dir, &self.files[1])
                    .unwrap()
                    .clone(),
            ]
        };
        let mut successful_deletions = 0;
        let mut errors = Vec::new();

        for file in files_to_delete {
            let filepath = build_full_path(&self.current_dir, &file);
            match remove_file(&filepath) {
                Ok(()) => successful_deletions += 1,
                Err(e) => errors.push(e),
            }
        }

        if !errors.is_empty() {
            self.notification = Notification::Error {
                msg: format!(
                    "Failed to delete {} files: {}",
                    errors.len(),
                    errors
                        .iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
                .into(),
            }
            .into()
        } else {
            self.notification = Notification::Success {
                msg: Lang::en_fmt("deleted", format_args!("{}", successful_deletions)).into(),
            }
            .into();
        }
        let position_id = get_position(&self.positions_map, &self.current_dir);
        let _ = self.reset_state(position_id.saturating_sub(1));
    }

    fn paste_files(&mut self) -> io::Result<()> {
        match &self.clipboard {
            Some(Clipboard::File { items, .. }) => {
                for file in items {
                    paste_file(file, &self.current_dir)?;
                }
                self.clipboard = None;
                let position_id = get_position(&self.positions_map, &self.current_dir);
                let _ = self.reset_state(position_id);
                self.notification = Notification::Success {
                    msg: Lang::en("pasted").into(),
                }
                .into();
                Ok(())
            }
            None => {
                self.notification = Notification::Warn {
                    msg: Lang::en("buffer_empty").into(),
                }
                .into();
                Ok(())
            }
        }
    }

    fn execute_file(&mut self, file_name: PathBuf) {
        let _ = exec(&self.config.common.editor, &[&file_name.to_string_lossy()]);
        self.from_external_app = true;
    }
}
