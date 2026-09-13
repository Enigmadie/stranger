use clap::Parser;
use std::{
    env,
    ffi::OsString,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use crate::app::utils::config_parser::default_config::Config;
use crate::app::utils::i18n::Lang;

pub mod default_config;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    editor: Option<String>,
    #[arg(long)]
    config_path: Option<PathBuf>,
}

pub fn load_config() -> Config {
    load_config_from(Args::parse())
}

fn load_config_from(args: Args) -> Config {
    let config_path = args.config_path.unwrap_or_else(default_config_path);

    let mut config = Config {
        config_path: config_path.clone(),
        ..Config::default()
    };
    let mut editor_from_file = false;

    if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(config_content) => match toml::from_str::<Config>(&config_content) {
                Ok(file_config) => {
                    config.common.editor = file_config.common.editor;
                    config.bookmarks = file_config.bookmarks;
                    editor_from_file = true;
                }
                Err(e) => {
                    eprintln!(
                        "{}",
                        Lang::en_fmt(
                            "config_parse_failed",
                            &[&config_path.to_string_lossy(), &e.to_string()]
                        )
                    );
                }
            },
            Err(e) => {
                eprintln!(
                    "{}",
                    Lang::en_fmt(
                        "config_read_failed",
                        &[&config_path.to_string_lossy(), &e.to_string()]
                    )
                );
            }
        }
    }

    if let Some(editor) = args.editor.or_else(|| {
        (!editor_from_file)
            .then(|| env::var("VISUAL").ok().or_else(|| env::var("EDITOR").ok()))
            .flatten()
    }) {
        config.common.editor = editor;
    }

    config
}

fn default_config_path() -> PathBuf {
    if let Some(path) = env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(path).join("stranger/config.toml");
    }
    #[cfg(target_os = "windows")]
    if let Some(path) = env::var_os("APPDATA") {
        return PathBuf::from(path).join("stranger/config.toml");
    }
    if let Some(home) = env::var_os("HOME") {
        let home = PathBuf::from(home);
        #[cfg(target_os = "macos")]
        return home.join("Library/Application Support/stranger/config.toml");
        #[cfg(not(target_os = "macos"))]
        return home.join(".config/stranger/config.toml");
    }
    PathBuf::from("config.toml")
}

fn validate_existing_config(path: &Path) -> io::Result<Option<fs::Permissions>> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let content = fs::read_to_string(path)?;
    toml::from_str::<Config>(&content).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            Lang::en_fmt(
                "config_malformed_refusing_overwrite",
                &[&path.to_string_lossy(), &error.to_string()],
            ),
        )
    })?;
    Ok(Some(metadata.permissions()))
}

fn atomic_write(
    path: &Path,
    content: &[u8],
    permissions: Option<fs::Permissions>,
) -> io::Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let file_name = path.file_name().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, Lang::en("config_path_invalid"))
    })?;

    for counter in 0_u32.. {
        let mut temp_name = OsString::from(".");
        temp_name.push(file_name);
        temp_name.push(format!(".tmp-{}-{counter}", std::process::id()));
        let temp_path = parent.join(temp_name);
        let mut temp_file = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
        {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        };

        let result = (|| {
            temp_file.write_all(content)?;
            temp_file.sync_all()?;
            if let Some(permissions) = permissions {
                fs::set_permissions(&temp_path, permissions)?;
            }
            drop(temp_file);
            fs::rename(&temp_path, path)
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temp_path);
        }
        return result;
    }
    unreachable!()
}

pub fn save_config(config: &Config) -> io::Result<()> {
    let path = if config.config_path.is_symlink() {
        config.config_path.canonicalize()?
    } else {
        config.config_path.clone()
    };
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    let permissions = validate_existing_config(&path)?;
    let toml_string = toml::to_string(config).map_err(io::Error::other)?;
    atomic_write(&path, toml_string.as_bytes(), permissions)
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
            config_path: Some(config_path.clone()),
        });

        save_config(&config).unwrap();

        let saved = fs::read_to_string(config_path).unwrap();
        assert!(saved.contains("editor = \"vim\""));
        assert!(!saved.contains("config_path"));
    }

    #[test]
    fn malformed_config_is_never_overwritten() {
        let temp = tempdir().unwrap();
        let config_path = temp.path().join("config.toml");
        let malformed = "[common\neditor = nope";
        fs::write(&config_path, malformed).unwrap();
        let config = Config {
            config_path: config_path.clone(),
            ..Config::default()
        };

        let error = save_config(&config).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert_eq!(fs::read_to_string(config_path).unwrap(), malformed);
    }

    #[test]
    fn config_save_replaces_valid_file_without_leaving_temp_files() {
        let temp = tempdir().unwrap();
        let config_path = temp.path().join("config.toml");
        let mut config = Config {
            config_path: config_path.clone(),
            ..Config::default()
        };
        save_config(&config).unwrap();
        config.common.editor = "vim".into();

        save_config(&config).unwrap();

        assert!(fs::read_to_string(config_path)
            .unwrap()
            .contains("editor = \"vim\""));
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 1);
    }

    #[test]
    fn default_config_is_outside_the_launch_directory_when_home_is_available() {
        let path = default_config_path();

        if env::var_os("HOME").is_some() || env::var_os("APPDATA").is_some() {
            assert!(path.ends_with("stranger/config.toml"));
            assert!(path.is_absolute());
        }
    }
}
