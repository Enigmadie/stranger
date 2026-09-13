pub struct Lang;

struct Message {
    id: &'static str,
    text: &'static str,
    placeholders: usize,
}

macro_rules! message {
    ($id:literal, $text:literal, $placeholders:literal) => {
        Message {
            id: $id,
            text: $text,
            placeholders: $placeholders,
        }
    };
}

const ENG: &[Message] = &[
    message!("in_buffer", "{} items copied in buffer.", 1),
    message!("deleted", "Deleted {} items!", 1),
    message!("moved", "Moved {} items!", 1),
    message!(
        "pasted_with_error",
        "Pasted {} items! Failed {} files: {}",
        3
    ),
    message!(
        "deleted_with_error",
        "Deleted {} items! Failed {} files: {}",
        3
    ),
    message!("moved_with_error", "Moved {} items! Failed {} files: {}", 3),
    message!("path_does_not_exist", "Path does not exist: {}", 1),
    message!("pasted", "Pasted {} items!", 1),
    message!("bookmark_added", "Bookmark added!", 0),
    message!("bookmark_deleted", "Bookmark deleted!", 0),
    message!(
        "bookmark_alias_exists",
        "Bookmark alias already exists: {}",
        1
    ),
    message!("bookmark_invalid", "Bookmark invalid!", 0),
    message!("path_invalid", "Path is invalid", 0),
    message!("items_not_found", "Files not found.", 0),
    message!("buffer_empty", "Buffer is empty.", 0),
    message!("items_not_pasted", "Files not pasted.", 0),
    message!("items_not_deleted", "Files not deleted.", 0),
    message!("insert_mode", "--INSERT--", 0),
    message!("visual_mode", "--VISUAL--", 0),
    message!("bookmarks_mode", "--BOOKMARKS--", 0),
    message!("no_matches", "No more matches for {}", 1),
    message!("matches", "Matches: {}", 1),
    message!("preview_read_error", "Error reading file", 0),
    message!(
        "preview_regular_files_only",
        "Preview is only available for regular files",
        0
    ),
    message!(
        "preview_binary_or_unsupported",
        "Binary or unsupported file",
        0
    ),
    message!("preview_empty", "Empty", 0),
    message!("directory_empty", "Empty directory", 0),
    message!("modal_add_file_title", "Add File", 0),
    message!("modal_rename_file_title", "Rename File", 0),
    message!("modal_add_bookmark_title", "Add New Bookmark Name", 0),
    message!("hint_bookmark_list", "Bookmark List", 0),
    message!("hint_add_bookmark", "Add Bookmark", 0),
    message!("hint_cut_files", "Cut Files", 0),
    message!(
        "hint_delete_to_trash",
        "Delete Files To Trash (On macOS, grant file access if prompted.)",
        0
    ),
    message!("hint_delete_permanently", "Delete Files Permanently", 0),
    message!(
        "hint_exit_current_directory",
        "Exit into current directory",
        0
    ),
    message!(
        "hint_exit_initial_directory",
        "Exit into initial directory",
        0
    ),
    message!("hint_column_key", "Key", 0),
    message!("hint_column_action", "Action", 0),
    message!("file_type_symlink", "symlink", 0),
    message!("file_type_special", "special", 0),
    message!(
        "cannot_delete_current_or_parent",
        "Cannot delete the current directory or one of its parents",
        0
    ),
    message!("file_update_failed", "Failed to update file: {}", 1),
    message!("files_delete_failed", "Failed to delete {} files: {}", 2),
    message!("file_name_required", "A single file name is required", 0),
    message!("destination_exists", "Destination already exists: {}", 1),
    message!(
        "cannot_copy_special_file",
        "Cannot copy special file: {}",
        1
    ),
    message!(
        "source_path_does_not_exist",
        "Source path does not exist: {}",
        1
    ),
    message!(
        "destination_directory_does_not_exist",
        "Destination directory does not exist: {}",
        1
    ),
    message!(
        "directory_transfer_into_itself",
        "Cannot copy or move a directory into itself",
        0
    ),
    message!("path_name_invalid", "Invalid path name", 0),
    message!("command_exited_with_status", "{} exited with status {}", 2),
    message!("editor_command_empty", "Editor command is empty", 0),
    message!(
        "config_parse_failed",
        "Failed to parse config file '{}': {}",
        2
    ),
    message!(
        "config_read_failed",
        "Failed to read config file '{}': {}",
        2
    ),
    message!(
        "config_malformed_refusing_overwrite",
        "Refusing to overwrite malformed config '{}': {}",
        2
    ),
    message!("config_path_invalid", "Invalid config path", 0),
    message!(
        "terminal_cleanup_failed",
        "Failed to cleanup terminal: {}",
        1
    ),
    message!("unknown_username", "unknown", 0),
    message!("unknown_hostname", "localhost", 0),
    message!("unknown_user_host", "unknown@localhost", 0),
];

impl Lang {
    pub fn en(key: &str) -> &'static str {
        ENG.iter()
            .find(|message| message.id == key)
            .map(|message| message.text)
            .unwrap_or("Unknown message")
    }

    pub fn en_fmt(key: &str, args: &[&str]) -> String {
        let message = ENG.iter().find(|message| message.id == key);
        let template = message
            .map(|message| message.text)
            .unwrap_or("Unknown message");
        let mut result = template.to_string();
        let placeholder_count = message.map(|message| message.placeholders).unwrap_or(0);

        if placeholder_count != args.len() {
            eprintln!(
                "Warning: Template '{}' has {} placeholders, but {} args provided: {:?}",
                template,
                placeholder_count,
                args.len(),
                args
            );
            return format!("Invalid format: {} (args: {:?})", template, args);
        }

        for arg in args {
            result = result.replacen("{}", arg, 1);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn message_ids_are_unique_and_placeholder_counts_are_valid() {
        let mut ids = HashSet::new();

        for message in ENG {
            assert!(
                ids.insert(message.id),
                "duplicate message ID: {}",
                message.id
            );
            assert_eq!(
                message.text.matches("{}").count(),
                message.placeholders,
                "invalid placeholder count for {}",
                message.id
            );
            assert_ne!(Lang::en(message.id), "Unknown message");
        }
    }
}
