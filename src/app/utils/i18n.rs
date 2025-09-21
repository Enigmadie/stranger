use std::collections::HashMap;

use once_cell::sync::Lazy;

pub struct Lang;

static ENG: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert("in_buffer", "{} items copied in buffer.");
    map.insert("deleted", "Deleted {} items!");
    map.insert("moved", "Moved {} items!");
    map.insert("pasted_with_error", "Pasted {} items! Failed {} files: {}");
    map.insert(
        "deleted_with_error",
        "Deleted {} items! Failed {} files: {}",
    );
    map.insert("moved_with_error", "Moved {} items! Failed {} files: {}");
    map.insert("path_does_not_exist", "Path does not exist: {}");
    map.insert("pasted", "Pasted {} items!");
    map.insert("bookmark_added", "Bookmark added!");
    map.insert("bookmark_deleted", "Bookmark deleted!");
    map.insert("bookmark_invalid", "Bookmark invalid!");
    map.insert("path_invalid", "Path is invalid");
    map.insert("items_not_found", "Files not found.");
    map.insert("buffer_empty", "Buffer is empty.");
    map.insert("items_not_pasted", "Files not pasted.");
    map.insert("items_not_deleted", "Files not deleted.");
    map.insert("insert_mode", "--INSERT--");
    map.insert("visual_mode", "--VISUAL--");
    map.insert("bookmarks_mode", "--BOOKMARKS--");
    map.insert("no_matches", "No more matches for {}");
    map.insert("matches", "Matches: {}");
    map
});

impl Lang {
    pub fn en(key: &str) -> &str {
        ENG.get(key).copied().unwrap_or("Unknown message")
    }

    pub fn en_fmt(key: &str, args: &[&str]) -> String {
        let template = Lang::en(key);
        let mut result = template.to_string();
        let placeholder_count = template.matches("{}").count();

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
