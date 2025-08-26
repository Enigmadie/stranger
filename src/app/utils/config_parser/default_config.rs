use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)] // config container
pub struct Config {
    pub common: CommonConfig,
    pub bookmarks: HashMap<String, PathBuf>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CommonConfig {
    pub editor: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            common: CommonConfig {
                editor: "nvim".to_string(),
            },
            bookmarks: HashMap::new(),
        }
    }
}
