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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::model::miller::entries::{FileEntry, FileVariant};
    use ratatui::widgets::List;

    fn render_row(is_current_column: bool) -> Buffer {
        let layout: Rc<[Rect]> = vec![
            Rect::new(0, 0, 1, 1),
            Rect::new(1, 0, 8, 1),
            Rect::new(9, 0, 1, 1),
            Rect::new(10, 0, 1, 1),
            Rect::new(11, 0, 1, 1),
        ]
        .into();
        let file = FileEntry {
            name: "file".into(),
            display_name: "file".into(),
            variant: FileVariant::File {
                size: None,
                permissions: None,
                last_modified: None,
                is_matched: false,
            },
        };
        let row = Row::build(
            layout,
            0,
            &file,
            RowContext {
                is_current_column,
                position_id: 0,
                col_width: 12,
                is_marked: false,
            },
            &Mode::Visual { anchor: 0 },
        );
        let area = Rect::new(0, 0, 12, 1);
        let mut buffer = Buffer::empty(area);
        List::new(vec![row]).render(area, &mut buffer);
        buffer
    }

    #[test]
    fn visual_cursor_highlight_is_limited_to_the_current_column() {
        assert!(render_row(true)
            .content
            .iter()
            .any(|cell| cell.bg == Color::Yellow));
        assert!(render_row(false)
            .content
            .iter()
            .all(|cell| cell.bg != Color::Yellow));
    }
}
