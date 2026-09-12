use std::{
    ffi::OsStr,
    fs::{File, Metadata, OpenOptions},
    io::{self, stdout},
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
};

#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::ffi::CString;

use crossterm::{
    cursor::Show,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen},
};
use std::io::Result as IoResult;

use crate::app::{
    cleanup_terminal,
    utils::{i18n::Lang, uniquify_path},
};

fn validate_file_name(name: &str) -> io::Result<&std::ffi::OsStr> {
    let mut components = Path::new(name).components();
    match (components.next(), components.next()) {
        (Some(Component::Normal(name)), None) => Ok(name),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "A single file name is required",
        )),
    }
}

#[cfg(target_os = "macos")]
fn rename_no_replace(source: &Path, destination: &Path) -> io::Result<()> {
    use std::os::unix::ffi::OsStrExt;

    let source = CString::new(source.as_os_str().as_bytes())
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    let destination = CString::new(destination.as_os_str().as_bytes())
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    // SAFETY: both pointers come from live CStrings and are valid for this call.
    let result =
        unsafe { libc::renamex_np(source.as_ptr(), destination.as_ptr(), libc::RENAME_EXCL) };
    if result == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(target_os = "linux")]
fn rename_no_replace(source: &Path, destination: &Path) -> io::Result<()> {
    use std::os::unix::ffi::OsStrExt;

    let source = CString::new(source.as_os_str().as_bytes())
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    let destination = CString::new(destination.as_os_str().as_bytes())
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    // SAFETY: both pointers come from live CStrings and are valid for this call.
    let result = unsafe {
        libc::renameat2(
            libc::AT_FDCWD,
            source.as_ptr(),
            libc::AT_FDCWD,
            destination.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn rename_no_replace(source: &Path, destination: &Path) -> io::Result<()> {
    if destination.try_exists()? {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("Destination already exists: {}", destination.display()),
        ));
    }
    std::fs::rename(source, destination)
}

pub fn rename_file(full_path: &Path, new_name: String) -> io::Result<()> {
    let parent_dir = full_path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file path"))?;

    let new_path = parent_dir.join(validate_file_name(&new_name)?);
    if new_path == full_path {
        return Ok(());
    }
    rename_no_replace(full_path, &new_path)
}

pub fn create_file(file_name: String, file_path: &Path) -> io::Result<()> {
    let full_path = file_path.join(validate_file_name(&file_name)?);
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(full_path)?;
    Ok(())
}

pub fn create_dir(dir_name: String, file_path: &Path) -> io::Result<()> {
    let full_path = file_path.join(validate_file_name(&dir_name)?);
    std::fs::create_dir(full_path)
}

pub fn copy_file_path(file_path: PathBuf) -> Result<PathBuf, io::Error> {
    file_path
        .symlink_metadata()
        .map(|_| file_path)
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, Lang::en("items_not_found")))
}

#[derive(Debug)]
pub enum MoveOutcome {
    Moved,
    CopiedButSourceRetained(io::Error),
}

fn reserve_unique_file(path: &Path) -> io::Result<(PathBuf, File)> {
    loop {
        let destination = uniquify_path(path);
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&destination)
        {
            Ok(file) => return Ok((destination, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
}

fn reserve_unique_directory(path: &Path) -> io::Result<PathBuf> {
    loop {
        let destination = uniquify_path(path);
        match std::fs::create_dir(&destination) {
            Ok(()) => return Ok(destination),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
}

#[cfg(unix)]
fn create_symlink(target: &Path, link: &Path, _source: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_symlink(target: &Path, link: &Path, source: &Path) -> io::Result<()> {
    if source.metadata()?.is_dir() {
        std::os::windows::fs::symlink_dir(target, link)
    } else {
        std::os::windows::fs::symlink_file(target, link)
    }
}

fn copy_symlink(source: &Path, destination: &Path) -> io::Result<()> {
    let target = std::fs::read_link(source)?;
    loop {
        let destination = uniquify_path(destination);
        match create_symlink(&target, &destination, source) {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
}

fn copy_directory_contents(source: &Path, destination: &Path) -> io::Result<()> {
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let metadata = source_path.symlink_metadata()?;
        let file_type = metadata.file_type();

        if file_type.is_symlink() {
            copy_symlink(&source_path, &destination_path)?;
        } else if file_type.is_file() {
            std::fs::copy(&source_path, &destination_path)?;
            std::fs::set_permissions(&destination_path, metadata.permissions())?;
        } else if file_type.is_dir() {
            std::fs::create_dir(&destination_path)?;
            copy_directory_contents(&source_path, &destination_path)?;
            std::fs::set_permissions(&destination_path, metadata.permissions())?;
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Cannot copy special file: {}", source_path.display()),
            ));
        }
    }
    Ok(())
}

fn prepare_transfer(src_path: &Path, dest_path: &Path) -> io::Result<(Metadata, PathBuf)> {
    let source_metadata = src_path.symlink_metadata().map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("Source path does not exist: {}", src_path.display()),
        )
    })?;
    let destination_metadata = dest_path.symlink_metadata().map_err(|error| {
        io::Error::new(
            error.kind(),
            format!(
                "Destination directory does not exist: {}",
                dest_path.display()
            ),
        )
    })?;
    if !destination_metadata.file_type().is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "Destination directory does not exist: {}",
                dest_path.display()
            ),
        ));
    }

    if source_metadata.file_type().is_dir() {
        let source = src_path.canonicalize()?;
        let destination = dest_path.canonicalize()?;
        if destination == source || destination.starts_with(&source) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Cannot copy or move a directory into itself",
            ));
        }
    }

    let destination = dest_path.join(
        src_path
            .file_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid path name"))?,
    );
    Ok((source_metadata, destination))
}

pub fn paste_file(src_path: &Path, dest_path: &Path) -> io::Result<()> {
    let (source_metadata, dest_dir) = prepare_transfer(src_path, dest_path)?;

    let source_type = source_metadata.file_type();
    if source_type.is_symlink() {
        copy_symlink(src_path, &dest_dir)?;
    } else if source_type.is_file() {
        let mut source = File::open(src_path)?;
        let permissions = source.metadata()?.permissions();
        let (destination_path, mut destination) = reserve_unique_file(&dest_dir)?;
        if let Err(error) = io::copy(&mut source, &mut destination)
            .and_then(|_| std::fs::set_permissions(&destination_path, permissions))
        {
            drop(destination);
            let _ = std::fs::remove_file(destination_path);
            return Err(error);
        }
    } else if source_type.is_dir() {
        let destination = reserve_unique_directory(&dest_dir)?;
        if let Err(error) = copy_directory_contents(src_path, &destination)
            .and_then(|()| std::fs::set_permissions(&destination, source_metadata.permissions()))
        {
            let _ = std::fs::remove_dir_all(destination);
            return Err(error);
        }
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            Lang::en("items_not_pasted"),
        ));
    }
    Ok(())
}

fn move_file_with<F>(src_path: &Path, dest_path: &Path, mut rename: F) -> io::Result<MoveOutcome>
where
    F: FnMut(&Path, &Path) -> io::Result<()>,
{
    let (source_metadata, destination) = prepare_transfer(src_path, dest_path)?;
    let source_type = source_metadata.file_type();
    if !source_type.is_symlink() && !source_type.is_file() && !source_type.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            Lang::en("items_not_pasted"),
        ));
    }

    loop {
        let destination = uniquify_path(&destination);
        match rename(src_path, &destination) {
            Ok(()) => return Ok(MoveOutcome::Moved),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) if error.kind() == io::ErrorKind::CrossesDevices => {
                paste_file(src_path, dest_path)?;
                return Ok(match remove_file(src_path) {
                    Ok(()) => MoveOutcome::Moved,
                    Err(error) => MoveOutcome::CopiedButSourceRetained(error),
                });
            }
            Err(error) => return Err(error),
        }
    }
}

pub fn move_file(src_path: &Path, dest_path: &Path) -> io::Result<MoveOutcome> {
    move_file_with(src_path, dest_path, rename_no_replace)
}

pub fn remove_file(path: &Path) -> io::Result<()> {
    let metadata = path.symlink_metadata().map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("Path does not exist: {}", path.display()),
        )
    })?;
    let file_type = metadata.file_type();

    if file_type.is_dir() && !file_type.is_symlink() {
        std::fs::remove_dir_all(path)?;
    } else {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

pub fn remove_file_to_trash(path: &Path) -> io::Result<()> {
    if path.symlink_metadata().is_err() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            Lang::en_fmt("path_does_not_exist", &[&path.to_string_lossy()]),
        ));
    }

    trash::delete(path).map_err(|error| io::Error::other(error.to_string()))
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

fn run_command(program: &str, args: &[&OsStr]) -> io::Result<()> {
    let status = Command::new(program)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "{program} exited with status {status}"
        )))
    }
}

pub fn exec(program: &str, args: &[&OsStr]) -> IoResult<()> {
    suspend_terminal()?;
    let command_result = run_command(program, args);
    let resume_result = resume_terminal();
    command_result.and(resume_result)
}

fn suspend_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), DisableMouseCapture, Show)
}

fn resume_terminal() -> io::Result<()> {
    enable_raw_mode()?;

    execute!(
        stdout(),
        EnterAlternateScreen,
        EnableMouseCapture,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn create_file_does_not_truncate_existing_file() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("note.md");
        fs::write(&path, "keep me").unwrap();

        let error = create_file("note.md".into(), temp.path()).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read_to_string(path).unwrap(), "keep me");
    }

    #[test]
    fn rename_does_not_replace_existing_file() {
        let temp = tempdir().unwrap();
        let source = temp.path().join("source.md");
        let destination = temp.path().join("destination.md");
        fs::write(&source, "source").unwrap();
        fs::write(&destination, "destination").unwrap();

        let error = rename_file(&source, "destination.md".into()).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read_to_string(source).unwrap(), "source");
        assert_eq!(fs::read_to_string(destination).unwrap(), "destination");
    }

    #[test]
    fn names_cannot_escape_the_current_directory() {
        let temp = tempdir().unwrap();

        assert_eq!(
            create_file("../outside".into(), temp.path())
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidInput
        );
        assert_eq!(
            create_dir("nested/directory".into(), temp.path())
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidInput
        );
    }

    #[test]
    fn directory_cannot_be_copied_into_itself_or_descendant() {
        let temp = tempdir().unwrap();
        let source = temp.path().join("source");
        let descendant = source.join("nested");
        fs::create_dir_all(&descendant).unwrap();

        assert_eq!(
            paste_file(&source, &source).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
        assert_eq!(
            paste_file(&source, &descendant).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
    }

    #[test]
    fn paste_preserves_colliding_file_and_directory() {
        let temp = tempdir().unwrap();
        let source_root = temp.path().join("source");
        let destination = temp.path().join("destination");
        fs::create_dir_all(source_root.join("folder")).unwrap();
        fs::create_dir(&destination).unwrap();
        fs::write(source_root.join("note.md"), "new").unwrap();
        fs::write(source_root.join("folder/nested.md"), "nested").unwrap();
        fs::write(destination.join("note.md"), "existing").unwrap();
        fs::create_dir(destination.join("folder")).unwrap();
        fs::write(destination.join("folder/existing.md"), "existing").unwrap();

        paste_file(&source_root.join("note.md"), &destination).unwrap();
        paste_file(&source_root.join("folder"), &destination).unwrap();

        assert_eq!(
            fs::read_to_string(destination.join("note.md")).unwrap(),
            "existing"
        );
        assert_eq!(
            fs::read_to_string(destination.join("note_.md")).unwrap(),
            "new"
        );
        assert_eq!(
            fs::read_to_string(destination.join("folder/existing.md")).unwrap(),
            "existing"
        );
        assert_eq!(
            fs::read_to_string(destination.join("folder_/nested.md")).unwrap(),
            "nested"
        );
    }

    #[test]
    fn move_renames_without_replacing_a_collision() {
        let temp = tempdir().unwrap();
        let source_root = temp.path().join("source");
        let destination = temp.path().join("destination");
        fs::create_dir(&source_root).unwrap();
        fs::create_dir(&destination).unwrap();
        let source = source_root.join("note.md");
        fs::write(&source, "new").unwrap();
        fs::write(destination.join("note.md"), "existing").unwrap();

        assert!(matches!(
            move_file(&source, &destination).unwrap(),
            MoveOutcome::Moved
        ));

        assert!(!source.exists());
        assert_eq!(
            fs::read_to_string(destination.join("note.md")).unwrap(),
            "existing"
        );
        assert_eq!(
            fs::read_to_string(destination.join("note_.md")).unwrap(),
            "new"
        );
    }

    #[cfg(unix)]
    #[test]
    fn move_falls_back_to_copy_and_delete_only_across_devices() {
        let temp = tempdir().unwrap();
        let destination = temp.path().join("destination");
        let source = temp.path().join("note.md");
        fs::create_dir(&destination).unwrap();
        fs::write(&source, "content").unwrap();

        let outcome = move_file_with(&source, &destination, |_, _| {
            Err(io::Error::from_raw_os_error(libc::EXDEV))
        })
        .unwrap();

        assert!(matches!(outcome, MoveOutcome::Moved));
        assert!(!source.exists());
        assert_eq!(
            fs::read_to_string(destination.join("note.md")).unwrap(),
            "content"
        );
    }

    #[test]
    fn move_does_not_copy_after_an_unrelated_rename_error() {
        let temp = tempdir().unwrap();
        let destination = temp.path().join("destination");
        let source = temp.path().join("note.md");
        fs::create_dir(&destination).unwrap();
        fs::write(&source, "content").unwrap();

        let error = move_file_with(&source, &destination, |_, _| {
            Err(io::Error::new(io::ErrorKind::PermissionDenied, "denied"))
        })
        .unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        assert!(source.exists());
        assert!(!destination.join("note.md").exists());
    }

    #[cfg(unix)]
    #[test]
    fn command_non_zero_exit_is_an_error() {
        let error = run_command("sh", &[OsStr::new("-c"), OsStr::new("exit 7")]).unwrap_err();

        assert!(error.to_string().contains("exited with status"));
        assert!(error.to_string().contains('7'));
    }

    #[cfg(unix)]
    #[test]
    fn paste_copies_symlink_without_following_it() {
        let temp = tempdir().unwrap();
        let source = temp.path().join("source");
        let destination = temp.path().join("destination");
        fs::create_dir(&source).unwrap();
        fs::create_dir(&destination).unwrap();
        fs::write(source.join("target"), "content").unwrap();
        std::os::unix::fs::symlink("target", source.join("link")).unwrap();

        paste_file(&source.join("link"), &destination).unwrap();

        let copied = destination.join("link");
        assert!(copied.symlink_metadata().unwrap().file_type().is_symlink());
        assert_eq!(fs::read_link(copied).unwrap(), PathBuf::from("target"));
    }

    #[cfg(unix)]
    #[test]
    fn deleting_directory_symlink_keeps_target() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("target");
        let link = temp.path().join("link");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("keep"), "content").unwrap();
        std::os::unix::fs::symlink(&target, &link).unwrap();

        remove_file(&link).unwrap();

        assert!(target.join("keep").exists());
        assert!(link.symlink_metadata().is_err());
    }

    #[cfg(unix)]
    #[test]
    fn paste_rejects_special_files() {
        use std::{ffi::CString, os::unix::ffi::OsStrExt};

        let temp = tempdir().unwrap();
        let destination = temp.path().join("destination");
        let fifo = temp.path().join("pipe");
        fs::create_dir(&destination).unwrap();
        let fifo_c = CString::new(fifo.as_os_str().as_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(fifo_c.as_ptr(), 0o600) }, 0);

        let error = paste_file(&fifo, &destination).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    }
}
