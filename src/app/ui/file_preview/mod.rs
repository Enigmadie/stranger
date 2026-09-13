use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
    time::SystemTime,
};

use once_cell::sync::Lazy;
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use crate::app::utils::i18n::Lang;

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileVersion {
    len: u64,
    modified: Option<SystemTime>,
}

#[derive(Debug, Clone)]
struct CachedPreview {
    version: FileVersion,
    lines: Vec<Line<'static>>,
}

#[derive(Debug, Default)]
pub struct PreviewCache {
    entries: HashMap<PathBuf, CachedPreview>,
}

impl PreviewCache {
    pub fn load(&mut self, file_path: &Path, max_bytes: usize) -> io::Result<Vec<Line<'static>>> {
        let metadata = file_path.symlink_metadata()?;
        if !metadata.file_type().is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Preview is only available for regular files",
            ));
        }
        let version = FileVersion {
            len: metadata.len(),
            modified: metadata.modified().ok(),
        };
        if let Some(cached) = self.entries.get(file_path) {
            if cached.version == version {
                return Ok(cached.lines.clone());
            }
        }

        let lines = highlight_file(file_path, max_bytes)?;
        if self.entries.len() >= 128 {
            self.entries.clear();
        }
        self.entries.insert(
            file_path.to_path_buf(),
            CachedPreview {
                version,
                lines: lines.clone(),
            },
        );
        Ok(lines)
    }

    pub fn invalidate(&mut self, file_path: &Path) {
        self.entries.remove(file_path);
    }
}
use syntect::{
    easy::HighlightLines,
    highlighting::{Style as SyntectStyle, ThemeSet},
    parsing::SyntaxSet,
};

pub fn highlight_file(file_path: &Path, max_bytes: usize) -> io::Result<Vec<Line<'static>>> {
    if !file_path.symlink_metadata()?.file_type().is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            Lang::en("preview_regular_files_only"),
        ));
    }

    let mut file = File::open(file_path)?;
    let mut bytes = Vec::new();
    file.by_ref()
        .take(max_bytes as u64)
        .read_to_end(&mut bytes)?;
    if bytes.contains(&0) {
        return Ok(vec![Line::from(Lang::en("preview_binary_or_unsupported"))]);
    }

    let content = match std::str::from_utf8(&bytes) {
        Ok(content) => content,
        Err(error) if error.error_len().is_none() => {
            std::str::from_utf8(&bytes[..error.valid_up_to()]).unwrap_or_default()
        }
        Err(_) => return Ok(vec![Line::from(Lang::en("preview_binary_or_unsupported"))]),
    };

    let content = content.replace('\t', "        ");

    let syntax = file_path
        .extension()
        .and_then(|extension| extension.to_str())
        .and_then(|extension| SYNTAX_SET.find_syntax_by_extension(extension))
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

    let mut h = HighlightLines::new(syntax, &THEME_SET.themes["base16-ocean.dark"]);

    let mut lines = Vec::new();
    for line in content.lines().take(50) {
        let ranges: Vec<(SyntectStyle, &str)> = h
            .highlight_line(line, &SYNTAX_SET)
            .unwrap_or_else(|_| vec![(SyntectStyle::default(), line)]);
        let spans: Vec<Span> = ranges
            .into_iter()
            .map(|(style, text)| {
                Span::styled(
                    text.to_string(),
                    Style::default().fg(Color::Rgb(
                        style.foreground.r,
                        style.foreground.g,
                        style.foreground.b,
                    )),
                )
            })
            .collect();
        lines.push(Line::from(spans));
    }

    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn previews_utf8_cyrillic_text() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("note.md");
        fs::write(&path, "Привет, мир\n").unwrap();

        let lines = highlight_file(&path, 2048).unwrap();
        let text = lines[0]
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();

        assert_eq!(text, "Привет, мир");
    }

    #[test]
    fn reports_invalid_utf8_as_binary() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("binary");
        fs::write(&path, [0xff, 0xfe]).unwrap();

        let lines = highlight_file(&path, 2048).unwrap();

        assert_eq!(lines[0].spans[0].content, "Binary or unsupported file");
    }

    #[cfg(unix)]
    #[test]
    fn rejects_fifo_without_opening_it() {
        use std::{ffi::CString, os::unix::ffi::OsStrExt};

        let temp = tempdir().unwrap();
        let path = temp.path().join("pipe");
        let path_c = CString::new(path.as_os_str().as_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(path_c.as_ptr(), 0o600) }, 0);

        let error = highlight_file(&path, 2048).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn preview_cache_reuses_and_invalidates_entries() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("note.md");
        fs::write(&path, "first").unwrap();
        let mut cache = PreviewCache::default();

        cache.load(&path, 2048).unwrap();
        assert_eq!(cache.entries.len(), 1);
        cache.load(&path, 2048).unwrap();
        assert_eq!(cache.entries.len(), 1);

        cache.invalidate(&path);
        assert!(cache.entries.is_empty());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn preview_accepts_non_utf8_paths() {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt};

        let temp = tempdir().unwrap();
        let path = temp
            .path()
            .join(OsString::from_vec(vec![b'n', b'o', b't', b'e', 0xff]));
        fs::write(&path, "content").unwrap();

        let lines = highlight_file(&path, 2048).unwrap();

        assert_eq!(lines[0].spans[0].content, "content");
    }
}
