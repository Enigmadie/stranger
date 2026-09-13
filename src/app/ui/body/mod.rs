use std::rc::Rc;

use ratatui::{
    layout::{Constraint, Direction, Flex, Layout},
    prelude::Rect,
    text::Line,
    widgets::{Block, List, ListItem, Paragraph, Widget},
};

use crate::app::{
    config::constants::ui::{COLUMN_PERCENTAGE, FIRST_COLUMN_PERCENTAGE},
    model::{
        file::{build_full_path, get_current_file},
        miller::positions::get_position,
    },
    state::State,
    ui::body::components::column_widget::{ColumnWidget, ColumnsWidget},
    utils::i18n::Lang,
};

pub mod bookmarks;
pub mod components;
pub mod row;
pub use row::{Row, RowContext};

pub struct Body;

const TARGET_POSITION_DOWN: usize = 6;

pub(crate) fn viewport_offset(item_count: usize, visible_height: usize, cursor: usize) -> usize {
    if visible_height == 0 || item_count <= visible_height {
        return 0;
    }
    let max_possible_offset = item_count.saturating_sub(visible_height);
    let upper_bound = visible_height.saturating_sub(TARGET_POSITION_DOWN);
    cursor.saturating_sub(upper_bound).min(max_possible_offset)
}

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

        let widgets: Vec<ColumnWidget<'a>> = state
            .files
            .iter()
            .enumerate()
            .map(|(col_id, dir)| {
                let is_parent_column = col_id == 0;
                let is_current_column = col_id == 1;
                let is_child_column = col_id >= 2;
                let is_current_or_child_column = is_current_column || is_child_column;
                let visible_height = layout[col_id].height.saturating_sub(2) as usize;

                let cursor = if is_current_column {
                    position_id
                } else if (is_child_column && col_id < state.dirs.len()) || is_parent_column {
                    state.dirs[col_id]
                        .dir_name
                        .as_ref()
                        .map_or(0, |name| get_position(&state.positions_map, name))
                } else {
                    0
                };

                let offset = if is_current_or_child_column {
                    viewport_offset(dir.len(), visible_height, cursor)
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

                if is_parent_column && dir.is_empty() {
                    // if parent dir is empty
                    ColumnWidget::Paragraph(Paragraph::new("").block(Block::default()))
                } else if is_child_column && dir.is_empty() {
                    let current_file =
                        get_current_file(&state.positions_map, &state.current_dir, &state.files[1]);
                    let is_current_column_and_selected_file =
                        current_file.is_some_and(|e| e.variant.is_regular_file());

                    let preview = if is_current_column_and_selected_file {
                        state.preview.clone()
                    } else {
                        vec![Line::from(Lang::en("preview_empty"))]
                    };

                    ColumnWidget::Paragraph(Paragraph::new(preview).block(Block::default()))
                } else if is_current_or_child_column && dir.is_empty() {
                    // if current or child dir are empty
                    ColumnWidget::Paragraph(
                        Paragraph::new(Lang::en("directory_empty")).block(Block::default()),
                    )
                } else {
                    let list_items: Vec<ListItem> = dir
                        .iter()
                        .skip(offset)
                        .take(visible_height)
                        .enumerate()
                        .map(|(row_id, file)| {
                            let is_marked = is_current_column
                                && state
                                    .marked
                                    .contains(&build_full_path(&state.current_dir, file));
                            Row::build(
                                Rc::clone(&row_layout),
                                row_id + offset,
                                file,
                                RowContext {
                                    is_current_column,
                                    position_id: cursor,
                                    col_width,
                                    is_marked,
                                },
                                &state.mode,
                            )
                        })
                        .collect();
                    ColumnWidget::List(List::new(list_items).block(Block::default()))
                }
            })
            .collect();

        ColumnsWidget::new(widgets, layout)
    }
}

#[cfg(test)]
mod tests {
    use super::viewport_offset;

    #[test]
    fn viewport_offset_handles_tiny_heights() {
        assert_eq!(viewport_offset(10, 0, 9), 0);
        assert_eq!(viewport_offset(10, 1, 9), 9);
        assert_eq!(viewport_offset(10, 5, 7), 5);
    }
}
