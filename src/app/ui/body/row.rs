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

pub struct RowContext {
    pub is_current_column: bool,
    pub position_id: usize,
    pub col_width: usize,
    pub is_marked: bool,
}

impl Row {
    pub fn build<'a>(
        row_layout: Rc<[Rect]>,
        row_id: usize,
        file: &'a FileEntry,
        context: RowContext,
        mode: &'a Mode,
    ) -> ListItem<'a> {
        let meta = match file.variant {
            FileVariant::Directory { len, .. } => len.map(|e| e.to_string()).unwrap_or_default(),
            FileVariant::File { size, .. } => size.map(format_bytes).unwrap_or_default(),
            FileVariant::Symlink { .. } => "link".to_string(),
            FileVariant::Special { .. } => "special".to_string(),
        };
        let meta_width = row_layout[2].width as usize;
        let name = file.display_name.as_str();

        let is_selected_row = row_id == context.position_id;

        let mut style = match file.variant {
            FileVariant::Directory { is_matched, .. } => {
                if is_selected_row {
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
                if is_selected_row {
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
            FileVariant::Symlink { is_matched } => {
                if is_selected_row {
                    Style::default().bg(Color::Cyan).fg(Color::Black).bold()
                } else if is_matched {
                    Style::default().fg(Color::Red).bold()
                } else {
                    Style::default().fg(Color::Cyan).bold()
                }
            }
            FileVariant::Special { is_matched } => {
                if is_selected_row {
                    Style::default().bg(Color::Magenta).fg(Color::Black).bold()
                } else if is_matched {
                    Style::default().fg(Color::Red).bold()
                } else {
                    Style::default().fg(Color::Magenta).bold()
                }
            }
        };

        if (matches!(mode, Mode::Visual { .. }) && context.is_current_column || context.is_marked)
            && is_selected_row
        {
            style = style.bg(Color::Yellow).fg(Color::Rgb(0, 0, 0));
        } else if context.is_marked {
            style = style.fg(Color::Yellow);
        }

        let padded_meta = if meta.len() >= meta_width {
            meta[..meta_width].to_string()
        } else {
            let pad = meta_width - meta.len();
            format!("{}{}", " ".repeat(pad), meta)
        };

        let mut buffer = Buffer::empty(Rect::new(0, 0, context.col_width as u16, 1));
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
