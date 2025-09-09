pub mod config_parser;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub mod fs;
pub mod i18n;
const KB: f64 = 1024.0;
const MB: f64 = KB * 1024.0;
const GB: f64 = MB * 1024.0;

pub fn format_bytes(bytes: u64) -> String {
    let b = bytes as f64;

    if b >= GB {
        format!("{} G", format_float_clean(bytes as f64 / GB))
    } else if b >= MB {
        format!("{} M", format_float_clean(bytes as f64 / MB))
    } else if b >= KB {
        format!("{} K", format_float_clean(bytes as f64 / KB))
    } else {
        format!("{} B", b)
    }
}

fn format_float_clean(n: f64) -> String {
    if n.fract() == 0.0 {
        format!("{:.0}", n)
    } else if (n * 10.0).fract() == 0.0 {
        format!("{:.1}", n)
    } else {
        format!("{:.2}", n)
    }
}

pub fn permissions_to_string(permissions: &Permissions) -> String {
    let mode = permissions.mode() & 0o777;
    let mut result = String::with_capacity(9);

    result.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    result.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    result.push(if mode & 0o100 != 0 { 'x' } else { '-' });

    result.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    result.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    result.push(if mode & 0o010 != 0 { 'x' } else { '-' });

    result.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    result.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    result.push(if mode & 0o001 != 0 { 'x' } else { '-' });

    result
}

fn uniquify_path(path: &Path) -> PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }

    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let ext = path.extension().and_then(|e| e.to_str());

    let mut counter = 0;
    loop {
        let candidate = match ext {
            Some(ext) if counter == 0 => parent.join(format!("{}_{}.{}", stem, "", ext)),
            Some(ext) => parent.join(format!("{}_{}.{}", stem, counter, ext)),
            None if counter == 0 => parent.join(format!("{}_{}", stem, "")),
            None => parent.join(format!("{}_{}", stem, counter)),
        };

        if !candidate.exists() {
            return candidate;
        }
        counter += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(2048), "2 K");
        assert_eq!(format_bytes(5_242_880), "5 M");
        assert_eq!(format_bytes(10_737_418_240), "10 G");
        assert_eq!(format_bytes(1_234), "1.21 K");
        assert_eq!(format_bytes(1_234_567), "1.18 M");
        assert_eq!(format_bytes(1_234_567_890), "1.15 G");
    }

    #[test]
    fn test_permissions_all() {
        let perms = Permissions::from_mode(0o777);
        assert_eq!(permissions_to_string(&perms), "rwxrwxrwx");
    }

    #[test]
    fn test_permissions_none() {
        let perms = Permissions::from_mode(0o000);
        assert_eq!(permissions_to_string(&perms), "---------");
    }

    #[test]
    fn test_permissions_mixed() {
        let perms = Permissions::from_mode(0o754);
        assert_eq!(permissions_to_string(&perms), "rwxr-xr--");
    }
    #[test]
    fn test_unique_when_not_exists() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("file.txt");

        let result = uniquify_path(&path);
        assert_eq!(result, path);
    }

    #[test]
    fn test_unique_with_extension() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("file.txt");

        File::create(&path).unwrap();

        let result = uniquify_path(&path);
        assert!(result.ends_with("file_1.txt"));
    }

    #[test]
    fn test_unique_without_extension() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("file");

        File::create(&path).unwrap();

        let result = uniquify_path(&path);
        assert!(result.ends_with("file_1"));
    }

    #[test]
    fn test_unique_multiple_collisions() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("data.log");

        File::create(&path).unwrap();
        File::create(dir.path().join("data_1.log")).unwrap();

        let result = uniquify_path(&path);
        assert!(result.ends_with("data_2.log"));
    }
}
