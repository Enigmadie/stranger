use std::collections::HashMap;

use once_cell::sync::Lazy;

pub struct Lang;

static ENG: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert("file_copied", "File copied!");
    map.insert("file_pasted", "File pasted!");
    map
});

impl Lang {
    pub fn en(key: &str) -> &str {
        ENG.get(key).copied().unwrap_or("Unknown message")
    }
}
