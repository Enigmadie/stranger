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
        marked: &'a [String],
    ) -> ListItem<'a> {
        let meta = match file.variant {
            FileVariant::Directory { len } => len.map(|e| e.to_string()).unwrap_or_default(),
            FileVariant::File { size } => size.map(format_bytes).unwrap_or_default(),
        };
        let meta_width = row_layout[2].width as usize;
        let name = file.name.as_str();

        let is_selected_column = is_current_column && row_id == position_id;
        let is_marked_in_visual_mode = is_current_column && marked.contains(&file.name);

        let mut style = match file.variant {
            FileVariant::Directory { .. } => {
                if is_selected_column {
                    Style::default()
                        .bg(Color::Blue)
                        .fg(Color::Rgb(0, 0, 0))
                        .bold()
                } else {
                    Style::default().fg(Color::Blue).bold()
                }
            }
            FileVariant::File { .. } => {
                if is_selected_column {
                    Style::default()
                        .bg(Color::White)
                        .fg(Color::Rgb(0, 0, 0))
                        .bold()
                } else {
                    Style::default().fg(Color::White).bold()
                }
            }
        };

        if is_marked_in_visual_mode {
            if is_selected_column {
                style = style.bg(Color::Yellow);
            } else {
                style = style.fg(Color::Yellow);
            }
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
}
