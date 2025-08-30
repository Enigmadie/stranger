use clap::Parser;
use std::fs;
use std::path::Path;

use crate::app::utils::config_parser::default_config::Config;

pub mod default_config;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    editor: Option<String>,
    #[arg(long, default_value = "config.toml")]
    config_path: String,
}

pub fn load_config() -> Config {
    let args = Args::parse();
    let config_path = &args.config_path;

    let mut config = Config::default();

    if Path::new(config_path).exists() {
        match fs::read_to_string(config_path) {
            Ok(config_content) => match toml::from_str::<Config>(&config_content) {
                Ok(file_config) => {
                    config.common.editor = file_config.common.editor;
                    config.bookmarks = file_config.bookmarks;
                }
                Err(e) => {
                    eprintln!("Failed to parse config file '{}': {}", config_path, e);
                }
            },
            Err(e) => {
                eprintln!("Failed to read config file '{}': {}", config_path, e);
            }
        }
    }

    if let Some(editor_arg) = args.editor {
        config.common.editor = editor_arg;
    }

    config
}

pub fn save_config(config: &Config) -> std::io::Result<()> {
    let args = Args::parse();
    let config_path = &args.config_path;

    let toml_string = toml::to_string(config).expect("Failed to serialize config to TOML");
    match fs::write(config_path, toml_string) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Failed to update config file '{}': {}", config_path, e);
            Err(e)
        }
    }
}
