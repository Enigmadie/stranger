use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};
use syntect::{
    easy::HighlightLines,
    highlighting::{Style as SyntectStyle, ThemeSet},
    parsing::SyntaxSet,
};

pub fn highlight_file(file_path: &Path, max_bytes: usize) -> io::Result<Vec<Line<'static>>> {
    if !file_path.symlink_metadata()?.file_type().is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Preview is only available for regular files",
        ));
    }

    let mut file = File::open(file_path)?;
    let mut bytes = Vec::new();
    file.by_ref()
        .take(max_bytes as u64)
        .read_to_end(&mut bytes)?;
    if bytes.contains(&0) {
        return Ok(vec![Line::from("Binary or unsupported file")]);
    }

    let content = match std::str::from_utf8(&bytes) {
        Ok(content) => content,
        Err(error) if error.error_len().is_none() => {
            std::str::from_utf8(&bytes[..error.valid_up_to()]).unwrap_or_default()
        }
        Err(_) => return Ok(vec![Line::from("Binary or unsupported file")]),
    };

    let content = content.replace('\t', "        ");

    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let syntax = ps
        .find_syntax_for_file(file_path)
        .unwrap_or_else(|_| Some(ps.find_syntax_plain_text()))
        .unwrap_or(ps.find_syntax_plain_text());

    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

    let mut lines = Vec::new();
    for line in content.lines().take(50) {
        let ranges: Vec<(SyntectStyle, &str)> = h
            .highlight_line(line, &ps)
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
}
