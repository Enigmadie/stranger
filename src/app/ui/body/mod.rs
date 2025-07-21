use std::rc::Rc;

use ratatui::{
    buffer::Buffer,
    layout::{self, Constraint, Direction, Flex, Layout},
    prelude::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, Widget},
};

use crate::app::{
    config::constants::ui::{COLUMN_PERCENTAGE, FIRST_COLUMN_PERCENTAGE},
    model::miller::{entries::FileVariant, positions::get_position},
    state::State,
    utils::format_bytes,
};

struct ColumnsWidget<'a> {
    lists: Vec<List<'a>>,
    layout: Rc<[Rect]>,
}

impl<'a> ColumnsWidget<'a> {
    fn new(lists: Vec<List<'a>>, layout: Rc<[Rect]>) -> Self {
        ColumnsWidget { lists, layout }
    }
}

impl<'a> Widget for ColumnsWidget<'a> {
    fn render(self, _area: Rect, buf: &mut Buffer) {
        for (i, list) in self.lists.into_iter().enumerate() {
            list.render(self.layout[i], buf);
        }
    }
}

pub struct Body;

impl Body {
    pub fn build<'a>(state: &'a State, area: Rect) -> impl Widget + 'a {
        let position_id = get_position(&state.positions_map, &state.current_dir);
        let constraints: Vec<Constraint> = state
            .files
            .iter()
            .enumerate()
            .map(|(i, _)| {
                if i == 0 {
                    Constraint::Percentage(FIRST_COLUMN_PERCENTAGE)
                } else {
                    Constraint::Percentage(COLUMN_PERCENTAGE)
                }
            })
            .collect();

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        let lists: Vec<List> = state
            .files
            .iter()
            .enumerate()
            .map(|(col_id, dir)| {
                let is_parent_column = col_id == 0;
                let is_current_column = col_id == 1;
                let is_child_column = col_id >= 2;

                let col_width = layout[col_id].width as usize;

                // Layout для строки: отступ слева, имя, метаданные, отступ справа
                let row_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Length(1),
                        Constraint::Percentage(80),
                        Constraint::Percentage(8), // адаптивный отступ
                        Constraint::Percentage(8),
                        Constraint::Length(1),
                    ])
                    .flex(Flex::SpaceBetween)
                    .split(Rect::new(0, 0, col_width as u16, 1));

                let name_width = row_layout[1].width as usize;
                let meta_width = row_layout[2].width as usize;
                let list_items: Vec<ListItem> = if is_parent_column && dir.is_empty() {
                    // if parent dir is empty
                    vec![]
                } else if (is_current_column || is_child_column) && dir.is_empty() {
                    // if current or child dir are empty
                    vec![ListItem::new("empty")]
                } else {
                    dir.iter()
                        .enumerate()
                        .map(|(row_id, file)| {
                            let meta = match file.variant {
                                FileVariant::Directory { len } => {
                                    len.map(|e| e.to_string()).unwrap_or_default()
                                }
                                FileVariant::File { size } => {
                                    size.map(format_bytes).unwrap_or_default()
                                }
                            };
                            let name = file.name.as_str();

                            let is_selected_column = is_current_column && row_id == position_id;

                            let style = match file.variant {
                                FileVariant::Directory { .. } => {
                                    if is_selected_column {
                                        Style::default().bg(Color::Blue).fg(Color::Black)
                                    } else {
                                        Style::default().fg(Color::Blue)
                                    }
                                }
                                FileVariant::File { .. } => {
                                    if is_selected_column {
                                        Style::default().bg(Color::White).fg(Color::Black)
                                    } else {
                                        Style::default().fg(Color::White)
                                    }
                                }
                            };

                            let padded_meta = if meta.len() >= meta_width {
                                meta[..meta_width].to_string() // усечём, если не влезает
                            } else {
                                let pad = meta_width - meta.len();
                                format!("{}{}", " ".repeat(pad), meta)
                            };

                            let mut buffer = Buffer::empty(Rect::new(0, 0, col_width as u16, 1));
                            for cell in buffer.content.iter_mut() {
                                cell.set_symbol(" "); // иначе .symbol() будет пустым
                                cell.set_style(style); // заливка по всей ширине
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
                        })
                        .collect()
                };
                List::new(list_items).block(Block::default())
            })
            .collect();

        ColumnsWidget::new(lists, layout)
    }
}
