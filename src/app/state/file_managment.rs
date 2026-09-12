use std::{
    io,
    path::{Path, PathBuf},
};

use crate::app::{
    model::{
        clipboard::{Clipboard, ClipboardAction},
        file::{build_full_path, get_current_file},
        miller::{
            columns::MillerColumns,
            positions::{get_position, update_dir_position},
        },
        notification::Notification,
    },
    state::{Bookmarks, HintBar, Mark, State},
    ui::modal::{ModalKind, UnderLineModalAction},
    utils::{
        fs::{
            copy_file_path, create_dir, create_file, exec, exec_shell_in, move_file, paste_file,
            remove_file, remove_file_to_trash, rename_file, MoveOutcome,
        },
        i18n::Lang,
    },
};

pub enum DeleteMode {
    Trash,
    Permanent,
}

pub trait FileManager {
    fn add_file(&mut self);
    fn rename_file(&mut self);
    fn copy_files(&mut self, action: ClipboardAction);
    fn paste_files(&mut self) -> io::Result<()>;
    fn delete_files(&mut self, mode: DeleteMode) -> io::Result<()>;
    fn commit_changes(&mut self);
    fn execute_file(&mut self, file_name: &Path) -> io::Result<()>;
    fn switch_to_current_dir(&self);
    fn toggle_hidden_files(&mut self) -> io::Result<()>;
}

fn selected_paths(state: &State<'_>) -> Option<Vec<PathBuf>> {
    if !state.marked.is_empty() {
        return Some(state.marked.clone());
    }

    get_current_file(&state.positions_map, &state.current_dir, &state.files[1])
        .map(|file| vec![build_full_path(&state.current_dir, file)])
}

fn deletion_paths(paths: Vec<PathBuf>, current_dir: &Path) -> io::Result<Vec<PathBuf>> {
    let current_dir = current_dir.canonicalize()?;
    let mut paths = paths
        .into_iter()
        .map(|path| {
            let is_symlink = path
                .symlink_metadata()
                .is_ok_and(|metadata| metadata.file_type().is_symlink());
            let canonical = if is_symlink {
                None
            } else {
                Some(path.canonicalize().unwrap_or_else(|_| path.clone()))
            };
            (path, canonical, is_symlink)
        })
        .collect::<Vec<_>>();
    paths.sort_by_key(|(path, canonical, _)| {
        canonical.as_ref().unwrap_or(path).components().count()
    });
    paths.dedup_by(|(left_path, left, _), (right_path, right, _)| {
        left_path == right_path || left.is_some() && left == right
    });

    if paths
        .iter()
        .filter_map(|(_, canonical, _)| canonical.as_ref())
        .any(|path| current_dir == *path || current_dir.starts_with(path))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Cannot delete the current directory or one of its parents",
        ));
    }

    let mut roots: Vec<(PathBuf, Option<PathBuf>, bool)> = Vec::new();
    for (path, canonical, is_symlink) in paths {
        let is_nested = roots
            .iter()
            .any(|(root_path, root_canonical, root_is_symlink)| {
                !root_is_symlink
                    && (path.starts_with(root_path)
                        || canonical.as_ref().is_some_and(|candidate| {
                            root_canonical
                                .as_ref()
                                .is_some_and(|root| candidate.starts_with(root))
                        }))
            });
        if !is_nested {
            roots.push((path, canonical, is_symlink));
        }
    }
    Ok(roots.into_iter().map(|(path, _, _)| path).collect())
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
        let action = match &self.modal_type {
            ModalKind::UnderLine { action } => *action,
            _ => return,
        };
        let result = match action {
            UnderLineModalAction::Add => {
                let is_dir = self
                    .input
                    .lines()
                    .last()
                    .is_some_and(|line| line.ends_with('/'));
                let result = if is_dir {
                    create_dir(input_value, &self.current_dir)
                } else {
                    create_file(input_value, &self.current_dir)
                };
                result.and_then(|()| self.reset_state_except_notifications(0))
            }
            UnderLineModalAction::Edit => {
                let current_file =
                    get_current_file(&self.positions_map, &self.current_dir, &self.files[1]);
                match current_file {
                    Some(file) => {
                        let full_path = build_full_path(&self.current_dir, file);
                        let position_id = get_position(&self.positions_map, &self.current_dir);
                        rename_file(&full_path, input_value)
                            .and_then(|()| self.reset_state_except_notifications(position_id))
                    }
                    None => Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("Failed to update file: {}", self.current_dir.display()),
                    )),
                }
            }
            UnderLineModalAction::Bookmarks => self.commit_new_bookmark(input_value),
        };

        if let Err(error) = result {
            self.notification = Some(Notification::Error {
                msg: error.to_string().into(),
            });
            return;
        }

        self.enter_normal_mode();
        self.setup_default_input();
    }

    fn copy_files(&mut self, action: ClipboardAction) {
        let Some(files_to_copy) = selected_paths(self) else {
            self.notification = Some(Notification::Warn {
                msg: Lang::en("items_not_found").into(),
            });
            if self.modal_type.is_hint_bar() {
                self.hide_hint_bar();
            }
            return;
        };
        let copied_filepaths: Result<Vec<PathBuf>, _> =
            files_to_copy.into_iter().map(copy_file_path).collect();

        match copied_filepaths {
            Ok(value) => {
                self.notification = Notification::Success {
                    msg: Lang::en_fmt("in_buffer", &[&value.len().to_string()]).into(),
                }
                .into();
                self.clipboard = Clipboard::File {
                    items: value,
                    action,
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
        if self.modal_type.is_hint_bar() {
            self.hide_hint_bar();
        }
    }

    fn delete_files(&mut self, mode: DeleteMode) -> io::Result<()> {
        let Some(files_to_delete) = selected_paths(self) else {
            self.notification = Some(Notification::Warn {
                msg: Lang::en("items_not_deleted").into(),
            });
            if self.modal_type.is_hint_bar() {
                self.hide_hint_bar();
            }
            return Ok(());
        };
        let files_to_delete = match deletion_paths(files_to_delete, &self.current_dir) {
            Ok(paths) => paths,
            Err(error) => {
                self.notification = Some(Notification::Error {
                    msg: error.to_string().into(),
                });
                if self.modal_type.is_hint_bar() {
                    self.hide_hint_bar();
                }
                return Ok(());
            }
        };
        let mut successful_deletions = 0;
        let mut errors = Vec::new();

        for filepath in files_to_delete {
            match mode {
                DeleteMode::Trash => match remove_file_to_trash(&filepath) {
                    Ok(()) => successful_deletions += 1,
                    Err(e) => errors.push(e),
                },
                DeleteMode::Permanent => match remove_file(&filepath) {
                    Ok(()) => successful_deletions += 1,
                    Err(e) => errors.push(e),
                },
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
                msg: Lang::en_fmt("deleted", &[&successful_deletions.to_string()]).into(),
            }
            .into();
        }
        self.clear_marks();
        let position_id = get_position(&self.positions_map, &self.current_dir);
        self.reset_state_except_notifications(position_id.saturating_sub(1))
    }

    fn paste_files(&mut self) -> io::Result<()> {
        match self.clipboard.as_ref() {
            Some(Clipboard::File { items, action }) => {
                let items = items.clone();
                let action = *action;
                let mut completed_files = 0;
                let mut retry_items = Vec::new();
                let mut errors = Vec::new();

                for file in items {
                    let result = if action == ClipboardAction::Cut {
                        move_file(&file, &self.current_dir).map(|outcome| match outcome {
                            MoveOutcome::Moved => None,
                            MoveOutcome::CopiedButSourceRetained(error) => Some(error),
                        })
                    } else {
                        paste_file(&file, &self.current_dir).map(|()| None)
                    };
                    match result {
                        Ok(None) => completed_files += 1,
                        Ok(Some(error)) => errors.push(error),
                        Err(err) => {
                            errors.push(err);
                            retry_items.push(file);
                        }
                    }
                }

                if !errors.is_empty() {
                    let lang_key_with_err = match action {
                        ClipboardAction::Copy => "pasted_with_error",
                        ClipboardAction::Cut => "moved_with_error",
                        ClipboardAction::Delete => "deleted_with_error",
                    };
                    self.notification = Notification::Warn {
                        msg: Lang::en_fmt(
                            lang_key_with_err,
                            &[
                                &completed_files.to_string(),
                                &errors.len().to_string(),
                                &errors
                                    .iter()
                                    .map(|e| e.to_string())
                                    .collect::<Vec<_>>()
                                    .join(", "),
                            ],
                        )
                        .into(),
                    }
                    .into();
                } else {
                    let lang_key = match action {
                        ClipboardAction::Copy => "pasted",
                        ClipboardAction::Cut => "moved",
                        ClipboardAction::Delete => "deleted",
                    };
                    self.notification = Notification::Success {
                        msg: Lang::en_fmt(lang_key, &[&completed_files.to_string()]).into(),
                    }
                    .into();
                }
                self.clipboard = (!retry_items.is_empty()).then_some(Clipboard::File {
                    items: retry_items,
                    action,
                });
                self.clear_marks();
                let position_id = get_position(&self.positions_map, &self.current_dir);
                self.reset_state_except_notifications(position_id)
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

    fn execute_file(&mut self, file_name: &Path) -> io::Result<()> {
        let editor_result = exec(&self.config.common.editor, &[file_name.as_os_str()]);
        self.from_external_app = true;
        let position_id = get_position(&self.positions_map, &self.current_dir);
        let refresh_result = self
            .reset_state_except_notifications(position_id)
            .and_then(|()| {
                let refreshed_position = position_id.min(self.files[1].len().saturating_sub(1));
                update_dir_position(
                    &mut self.positions_map,
                    &self.current_dir,
                    refreshed_position,
                );
                if refreshed_position != position_id {
                    self.reset_state_except_notifications(refreshed_position)?;
                }
                Ok(())
            });
        editor_result.and(refresh_result)
    }

    fn switch_to_current_dir(&self) {
        let _ = exec_shell_in(&self.current_dir);
    }

    fn toggle_hidden_files(&mut self) -> io::Result<()> {
        self.show_hidden_files = !self.show_hidden_files;
        let position_id = get_position(&self.positions_map, &self.current_dir);
        self.reset_state_except_notifications(position_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::test_utils::{create_test_state, create_test_state_at};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn copy_in_empty_directory_does_not_panic_or_fill_clipboard() {
        let mut state = create_test_state();
        state.files[1].clear();

        state.copy_files(ClipboardAction::Copy);

        assert!(state.clipboard.is_none());
        assert!(matches!(
            state.notification,
            Some(Notification::Warn { .. })
        ));
    }

    #[test]
    fn delete_in_empty_directory_does_not_panic() {
        let mut state = create_test_state();
        state.files[1].clear();

        state.delete_files(DeleteMode::Permanent).unwrap();

        assert!(matches!(
            state.notification,
            Some(Notification::Warn { .. })
        ));
    }

    #[test]
    fn deletion_rejects_the_current_directory_and_its_parents() {
        let temp = tempdir().unwrap();
        let current = temp.path().join("parent/child");
        fs::create_dir_all(&current).unwrap();

        assert!(deletion_paths(vec![current.clone()], &current).is_err());
        assert!(deletion_paths(vec![temp.path().join("parent")], &current).is_err());
        assert!(current.exists());
    }

    #[test]
    fn deletion_collapses_nested_paths_to_their_selected_root() {
        let temp = tempdir().unwrap();
        let current = temp.path().join("current");
        let parent = temp.path().join("selected");
        let child = parent.join("child");
        fs::create_dir_all(&current).unwrap();
        fs::create_dir_all(&child).unwrap();

        let paths = deletion_paths(vec![child, parent.clone()], &current).unwrap();

        assert_eq!(paths, vec![parent]);
    }

    #[cfg(unix)]
    #[test]
    fn deletion_does_not_resolve_symlink_to_current_directory() {
        let temp = tempdir().unwrap();
        let current = temp.path().join("current");
        let link = temp.path().join("current-link");
        fs::create_dir(&current).unwrap();
        std::os::unix::fs::symlink(&current, &link).unwrap();

        let paths = deletion_paths(vec![link.clone()], &current).unwrap();

        assert_eq!(paths, vec![link]);
    }

    #[test]
    fn partial_paste_keeps_failed_items_in_clipboard() {
        let temp = tempdir().unwrap();
        let destination = temp.path().join("destination");
        let source = temp.path().join("source.md");
        let missing = temp.path().join("missing.md");
        fs::create_dir(&destination).unwrap();
        fs::write(&source, "source").unwrap();
        let mut state = create_test_state_at(&destination).unwrap();
        state.clipboard = Some(Clipboard::File {
            items: vec![source, missing.clone()],
            action: ClipboardAction::Copy,
        });

        state.paste_files().unwrap();

        assert!(destination.join("source.md").exists());
        assert!(matches!(
            state.clipboard,
            Some(Clipboard::File { ref items, .. }) if items == &[missing]
        ));
    }

    #[test]
    fn stale_mark_does_not_block_deleting_existing_marks() {
        let temp = tempdir().unwrap();
        let current = temp.path().join("current");
        let existing = temp.path().join("existing.md");
        let missing = temp.path().join("missing.md");
        fs::create_dir(&current).unwrap();
        fs::write(&existing, "existing").unwrap();
        let mut state = create_test_state_at(&current).unwrap();
        state.marked = vec![missing, existing.clone()];

        state.delete_files(DeleteMode::Permanent).unwrap();

        assert!(!existing.exists());
        assert!(matches!(
            state.notification,
            Some(Notification::Error { .. })
        ));
    }
}
