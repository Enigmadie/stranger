use std::{
    fs::File,
    io::{self, Read},
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

pub fn highlight_file(file_path: &str, max_bytes: usize) -> io::Result<Vec<Line<'static>>> {
    if is_binary_file(file_path)? {
        return Ok(vec![Line::from("Binary or unsupported file")]);
    }
    let file = File::open(file_path)?;
    let mut content = String::new();
    file.take(max_bytes as u64).read_to_string(&mut content)?;

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

fn is_binary_file(file_path: &str) -> io::Result<bool> {
    let mut file = File::open(file_path)?;
    let mut buffer = [0u8; 1024];
    let bytes_read = file.read(&mut buffer)?;
    Ok(buffer[..bytes_read]
        .iter()
        .any(|&b| b == 0 || !b.is_ascii()))
}
