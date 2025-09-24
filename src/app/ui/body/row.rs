use std::rc::Rc;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{ListItem, Widget},
};

use crate::app::{
    model::miller::entries::{FileEntry, FileVariant},
    state::Mode,
    utils::format_bytes,
};

pub struct Row {}

impl Row {
    pub fn build<'a>(
        row_layout: Rc<[Rect]>,
        row_id: usize,
        file: &'a FileEntry,
        is_current_column: bool,
        position_id: usize,
        col_width: usize,
        marked: &'a [FileEntry],
        mode: &'a Mode,
    ) -> ListItem<'a> {
        let meta = match file.variant {
            FileVariant::Directory { len, .. } => len.map(|e| e.to_string()).unwrap_or_default(),
            FileVariant::File { size, .. } => size.map(format_bytes).unwrap_or_default(),
        };
        let meta_width = row_layout[2].width as usize;
        let name = file.name.as_str();

        let is_selected_column = row_id == position_id;
        let is_marked = is_current_column && marked.iter().any(|f| f.name == file.name);

        let mut style = match file.variant {
            FileVariant::Directory { is_matched, .. } => {
                if is_selected_column {
                    Style::default()
                        .bg(Color::Blue)
                        .fg(Color::Rgb(0, 0, 0))
                        .bold()
                } else if is_matched {
                    Style::default().fg(Color::Red).bold()
                } else {
                    Style::default().fg(Color::Blue).bold()
                }
            }
            FileVariant::File { is_matched, .. } => {
                if is_selected_column {
                    Style::default()
                        .bg(Color::White)
                        .fg(Color::Rgb(0, 0, 0))
                        .bold()
                } else if is_matched {
                    Style::default().fg(Color::Red).bold()
                } else {
                    Style::default().fg(Color::White).bold()
                }
            }
        };

        if (matches!(mode, Mode::Visual { .. }) || is_marked) && is_selected_column {
            style = style.bg(Color::Yellow).fg(Color::Rgb(0, 0, 0));
        } else if is_marked {
            style = style.fg(Color::Yellow);
        }

        let padded_meta = if meta.len() >= meta_width {
            meta[..meta_width].to_string()
        } else {
            let pad = meta_width - meta.len();
            format!("{}{}", " ".repeat(pad), meta)
        };

        let mut buffer = Buffer::empty(Rect::new(0, 0, col_width as u16, 1));
        for cell in buffer.content.iter_mut() {
            cell.set_symbol(" ");
            cell.set_style(style);
        }
        Span::styled(name, style).render(row_layout[1], &mut buffer);
        Span::styled(padded_meta, style).render(row_layout[3], &mut buffer);

        let line = Line::from(
            buffer
                .content
                .iter()
                .map(|c| Span::styled(c.symbol().to_string(), c.style()))
                .collect::<Vec<_>>(),
        );

        ListItem::new(line).style(style)
    }

    pub fn bookmarks_build<'a>(
        row_id: usize,
        file: String,
        is_current_column: bool,
        position_id: usize,
    ) -> ListItem<'a> {
        let is_selected_column = is_current_column && row_id == position_id;
        let style = if is_selected_column {
            Style::default()
                .bg(Color::LightCyan)
                .fg(Color::Rgb(0, 0, 0))
                .bold()
        } else {
            Style::default().fg(Color::Gray).bold()
        };

        let line = Line::from(Span::styled(file, style));

        ListItem::new(line).style(style)
    }
}
