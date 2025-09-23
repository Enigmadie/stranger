use fs_extra::dir::{self, CopyOptions};
use std::{
    io::{self, stdout},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use crossterm::{
    cursor::Show,
    execute,
    terminal::{enable_raw_mode, Clear, ClearType, EnterAlternateScreen},
};
use std::io::Result as IoResult;

use crate::app::{
    cleanup_terminal,
    utils::{i18n::Lang, uniquify_path},
};

pub fn rename_file(full_path: &PathBuf, new_name: String) -> io::Result<()> {
    let parent_dir = full_path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file path"))?;

    let new_path = parent_dir.join(&new_name);
    std::fs::rename(full_path, new_path)
}

pub fn create_file(file_name: String, file_path: &PathBuf) -> io::Result<()> {
    let full_path = PathBuf::from(file_path).join(file_name);

    let parent_dir = full_path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file path"))?;

    if !parent_dir.exists() {
        std::fs::create_dir_all(parent_dir)?;
    }

    std::fs::File::create(&full_path)?;
    Ok(())
}

pub fn create_dir(dir_name: String, file_path: &PathBuf) -> io::Result<()> {
    let full_path = PathBuf::from(file_path).join(dir_name);

    let parent_dir = full_path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file path"))?;

    if !parent_dir.exists() {
        std::fs::create_dir_all(parent_dir)?;
    }

    std::fs::create_dir_all(&full_path)?;
    Ok(())
}

pub fn copy_file_path(file_path: PathBuf) -> Result<PathBuf, io::Error> {
    let path = PathBuf::from(&file_path);
    if Path::new(&file_path).exists() {
        Ok(path)
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            Lang::en("items_not_found"),
        ))
    }
}

pub fn paste_file(src_path: &PathBuf, dest_path: &Path) -> io::Result<()> {
    if !dest_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Source path does not exist: {}", src_path.display()),
        ));
    }

    let dest_dir = dest_path.join(
        src_path
            .file_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid path name"))?,
    );

    let uniq_dest = uniquify_path(&dest_dir);

    if src_path.is_file() {
        std::fs::copy(src_path, uniq_dest)?;
    } else if src_path.is_dir() {
        let options = CopyOptions::new().overwrite(true).copy_inside(true);
        let _ = dir::copy(src_path, uniq_dest, &options);
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            Lang::en("items_not_pasted"),
        ));
    }
    Ok(())
}

pub fn remove_file(path: &PathBuf) -> io::Result<()> {
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            Lang::en_fmt("path_does_not_exist", &[&path.to_string_lossy()]),
        ));
    }

    if path.is_file() {
        std::fs::remove_file(path)?;
    } else if path.is_dir() {
        std::fs::remove_dir_all(path)?;
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            Lang::en("items_not_deleted"),
        ));
    }
    Ok(())
}

pub fn remove_file_to_trash(path: &PathBuf) -> io::Result<()> {
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            Lang::en_fmt("path_does_not_exist", &[&path.to_string_lossy()]),
        ));
    }

    match trash::delete(path) {
        Ok(()) => Ok(()),
        Err(_) => remove_file(path),
    }
}

pub fn whoami_info() -> io::Result<String> {
    let username = Command::new("whoami")
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_else(|_| String::from("unknown"));

    let hostname = Command::new("scutil")
        .arg("--get")
        .arg("LocalHostName")
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_else(|_| String::from("localhost"));

    Ok(format!("{}@{}", username, hostname))
}

pub fn exec(program: &String, arg: &[&str]) -> IoResult<()> {
    let _ = Command::new(program)
        .args(arg)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("couldn't run program");

    enable_raw_mode()?;

    execute!(
        stdout(),
        EnterAlternateScreen,
        Clear(ClearType::All),
        Show,
        crossterm::cursor::MoveTo(0, 0)
    )?;

    Ok(())
}

#[cfg(unix)]
pub fn exec_shell_in(dir: &PathBuf) -> io::Result<()> {
    use std::os::unix::process::CommandExt;

    cleanup_terminal()?;

    std::env::set_current_dir(dir)?;

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    let err = std::process::Command::new(&shell)
        .arg("-l")
        .arg("-i")
        .exec();

    Err(err)
}
