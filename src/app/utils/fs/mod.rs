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

use crate::app::utils::i18n::Lang;

pub fn rename_file(path: &PathBuf, new_value: String) -> io::Result<()> {
    std::fs::rename(path, new_value)
}

pub fn create_file(file_name: String) -> io::Result<()> {
    let _ = std::fs::File::create(file_name);
    Ok(())
}

pub fn create_dir(dir_name: String) -> io::Result<()> {
    let _ = std::fs::create_dir(dir_name);
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

    if src_path.is_file() {
        std::fs::copy(src_path, &dest_dir)?;
    } else if src_path.is_dir() {
        let options = CopyOptions::new().overwrite(true).copy_inside(true);
        let _ = dir::copy(src_path, &dest_dir, &options);
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
            format!("Path does not exist: {}", path.display()),
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
