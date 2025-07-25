use std::{
    io::{self, stdout},
    path::PathBuf,
    process::{Command, Stdio},
};

use crossterm::{
    cursor::Show,
    execute,
    terminal::{enable_raw_mode, Clear, ClearType, EnterAlternateScreen},
};
use std::io::Result as IoResult;

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
