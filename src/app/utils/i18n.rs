use std::collections::HashMap;

use once_cell::sync::Lazy;

pub struct Lang;

static ENG: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert("copied", "Copied!");
    map.insert("pasted", "Pasted!");
    map.insert("items_not_found", "Files not found.");
    map.insert("buffer_empty", "Buffer is empty");
    map.insert("items_not_pasted", "Files not pasted.");
    map
});

impl Lang {
    pub fn en(key: &str) -> &str {
        ENG.get(key).copied().unwrap_or("Unknown message")
    }
}
