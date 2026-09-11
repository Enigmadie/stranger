use clap::Parser;
use std::fs;
use std::path::PathBuf;

use crate::app::utils::config_parser::default_config::Config;

pub mod default_config;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    editor: Option<String>,
    #[arg(long, default_value = "config.toml")]
    config_path: PathBuf,
}

pub fn load_config() -> Config {
    load_config_from(Args::parse())
}

fn load_config_from(args: Args) -> Config {
    let config_path = args.config_path;

    let mut config = Config {
        config_path: config_path.clone(),
        ..Config::default()
    };

    if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(config_content) => match toml::from_str::<Config>(&config_content) {
                Ok(file_config) => {
                    config.common.editor = file_config.common.editor;
                    config.bookmarks = file_config.bookmarks;
                }
                Err(e) => {
                    eprintln!(
                        "Failed to parse config file '{}': {}",
                        config_path.display(),
                        e
                    );
                }
            },
            Err(e) => {
                eprintln!(
                    "Failed to read config file '{}': {}",
                    config_path.display(),
                    e
                );
            }
        }
    }

    if let Some(editor_arg) = args.editor {
        config.common.editor = editor_arg;
    }

    config
}

pub fn save_config(config: &Config) -> std::io::Result<()> {
    let toml_string = toml::to_string(config).map_err(std::io::Error::other)?;
    fs::write(&config.config_path, toml_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn config_path_is_runtime_state_and_is_used_for_saving() {
        let temp = tempdir().unwrap();
        let config_path = temp.path().join("custom.toml");
        let config = load_config_from(Args {
            editor: Some("vim".into()),
            config_path: config_path.clone(),
        });

        save_config(&config).unwrap();

        let saved = fs::read_to_string(config_path).unwrap();
        assert!(saved.contains("editor = \"vim\""));
        assert!(!saved.contains("config_path"));
    }
}
