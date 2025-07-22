use std::rc::Rc;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Flex, Layout},
    prelude::Rect,
    widgets::{Block, List, ListItem, Widget},
};

use crate::app::{
    config::constants::ui::{COLUMN_PERCENTAGE, FIRST_COLUMN_PERCENTAGE},
    model::miller::positions::get_position,
    state::State,
};

pub mod row;
pub use row::Row;

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
                let is_current_or_child_column = is_current_column || is_child_column;
                let visible_height = layout[col_id].height.saturating_sub(2) as usize;

                const TARGET_POSITION_DOWN: usize = 6;

                let cursor = if is_current_column {
                    position_id
                } else if is_child_column && col_id < state.dirs.len() {
                    state.dirs[col_id]
                        .dir_name
                        .as_ref()
                        .map_or(0, |name| get_position(&state.positions_map, name))
                } else {
                    0
                };

                let offset = if is_current_or_child_column && dir.len() > visible_height {
                    let max_possible_offset = dir.len().saturating_sub(visible_height);
                    let upper_bound = visible_height - TARGET_POSITION_DOWN; // 30 - 6 = 24;
                    if cursor >= upper_bound {
                        // 50 >= 24
                        (cursor - upper_bound).min(max_possible_offset)
                    } else {
                        0
                    }
                } else {
                    0
                };

                let col_width = layout[col_id].width as usize;

                let row_layout = Rc::new(
                    Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Length(1),
                            Constraint::Percentage(76),
                            Constraint::Percentage(12),
                            Constraint::Percentage(12),
                            Constraint::Length(1),
                        ])
                        .flex(Flex::SpaceBetween)
                        .split(Rect::new(0, 0, col_width as u16, 1)),
                );

                let list_items: Vec<ListItem> = if is_parent_column && dir.is_empty() {
                    // if parent dir is empty
                    vec![]
                } else if is_current_or_child_column && dir.is_empty() {
                    // if current or child dir are empty
                    vec![ListItem::new("empty")]
                } else {
                    dir.iter()
                        .skip(offset)
                        .take(visible_height)
                        .enumerate()
                        .map(|(row_id, file)| {
                            Row::build(
                                Rc::clone(&row_layout),
                                row_id + offset,
                                file,
                                is_current_or_child_column,
                                cursor,
                                col_width,
                            )
                        })
                        .collect()
                };
                List::new(list_items).block(Block::default())
            })
            .collect();

        ColumnsWidget::new(lists, layout)
    }
}
