use std::collections::HashMap;

use once_cell::sync::Lazy;

pub struct Lang;

static ENG: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert("copied", "Copied {} items!");
    map.insert("deleted", "Deleted {} items!");
    map.insert("pasted", "Pasted!");
    map.insert("bookmark_added", "Bookmark added!");
    map.insert("items_not_found", "Files not found.");
    map.insert("buffer_empty", "Buffer is empty");
    map.insert("items_not_pasted", "Files not pasted.");
    map.insert("insert_mode", "--INSERT--");
    map.insert("visual_mode", "--VISUAL--");
    map
});

impl Lang {
    pub fn en(key: &str) -> &str {
        ENG.get(key).copied().unwrap_or("Unknown message")
    }

    pub fn en_fmt(key: &str, args: std::fmt::Arguments<'_>) -> String {
        let template = Lang::en(key);
        if template.contains("{}") {
            let mut result = String::new();
            std::fmt::write(&mut result, args).unwrap();
            template.replace("{}", &result)
        } else {
            template.to_string()
        }
    }
}
