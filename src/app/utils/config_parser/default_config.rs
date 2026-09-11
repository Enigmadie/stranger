use std::path::PathBuf;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)] // config container
pub struct Config {
    #[serde(skip)]
    pub config_path: PathBuf,
    pub common: CommonConfig,
    pub bookmarks: IndexMap<String, PathBuf>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CommonConfig {
    pub editor: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            config_path: PathBuf::from("config.toml"),
            common: CommonConfig {
                editor: "nvim".to_string(),
            },
            bookmarks: IndexMap::new(),
        }
    }
}
