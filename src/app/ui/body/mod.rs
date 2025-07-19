use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout},
    prelude::Rect,
    style::{Color, Style},
    widgets::{Block, List, ListItem, Widget},
};

use crate::app::{
    config::constants::ui::{COLUMN_PERCENTAGE, FIRST_COLUMN_PERCENTAGE},
    model::{file::FileVariant, miller::positions::get_position},
    state::State,
    utils::format_bytes,
};

struct ColumnsWidget<'a> {
    lists: Vec<List<'a>>,
}

impl<'a> ColumnsWidget<'a> {
    fn new(lists: Vec<List<'a>>) -> Self {
        ColumnsWidget { lists }
    }
}

impl<'a> Widget for ColumnsWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let constraints: Vec<Constraint> = self
            .lists
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

        for (i, list) in self.lists.into_iter().enumerate() {
            list.render(layout[i], buf);
        }
    }
}

pub struct Body;

impl Body {
    pub fn build<'a>(state: &'a State, _area: Rect) -> impl Widget + 'a {
        let position_id = get_position(&state.positions_map, &state.current_dir);
        let lists: Vec<List> = state
            .files
            .iter()
            .enumerate()
            .map(|(col_id, dir)| {
                let is_parent_column = col_id == 0;
                let is_current_column = col_id == 1;
                let is_child_column = col_id >= 2;

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
                            let mut list_item = ListItem::new(format!(
                                "{:<30} {:>24}",
                                file.name.as_str(),
                                match file.variant {
                                    FileVariant::Directory { len } => len.to_string(),
                                    FileVariant::File { size } => format_bytes(size),
                                }
                            ));
                            let is_selected_column = is_current_column && row_id == position_id;

                            list_item = match file.variant {
                                FileVariant::Directory { .. } => {
                                    if is_selected_column {
                                        list_item.style(
                                            Style::default().bg(Color::Blue).fg(Color::Black),
                                        )
                                    } else {
                                        list_item.style(Style::default().fg(Color::Blue))
                                    }
                                }
                                FileVariant::File { .. } => {
                                    if is_selected_column {
                                        list_item.style(
                                            Style::default().bg(Color::White).fg(Color::Black),
                                        )
                                    } else {
                                        list_item.style(Style::default().fg(Color::White))
                                    }
                                }
                            };

                            list_item
                        })
                        .collect()
                };
                List::new(list_items).block(Block::default())
            })
            .collect();

        ColumnsWidget::new(lists)
    }
}
