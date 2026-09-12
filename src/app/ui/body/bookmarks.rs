use std::path::PathBuf;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::Rect,
    widgets::{Block, List, ListItem, Widget},
};

use crate::app::{
    state::State,
    ui::body::{
        components::column_widget::{ColumnWidget, ColumnsWidget},
        viewport_offset, Row,
    },
};

pub struct Bookmarks;

impl Bookmarks {
    pub fn build<'a>(state: &'a State, position_id: usize, area: Rect) -> impl Widget + 'a {
        let (names, paths): (Vec<&String>, Vec<&PathBuf>) = state.config.bookmarks.iter().unzip();

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);

        let cursor = position_id;
        let visible_height = layout[0].height.saturating_sub(2) as usize;
        let offset = viewport_offset(names.len(), visible_height, cursor);

        let list_names: Vec<ListItem> = names
            .iter()
            .skip(offset)
            .take(visible_height)
            .enumerate()
            .map(|(row_id, file)| {
                Row::bookmarks_build(row_id + offset, file.to_string(), true, cursor)
            })
            .collect();

        let path_strings: Vec<String> = paths
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();

        let list_paths: Vec<ListItem> = path_strings
            .iter()
            .skip(offset)
            .take(visible_height)
            .enumerate()
            .map(|(row_id, path)| {
                Row::bookmarks_build(row_id + offset, path.to_string(), false, cursor)
            })
            .collect();

        ColumnsWidget::new(
            vec![
                ColumnWidget::List(List::new(list_names).block(Block::default())),
                ColumnWidget::List(List::new(list_paths).block(Block::default())),
            ],
            layout,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::test_utils::create_test_state;
    use ratatui::{buffer::Buffer, style::Color, widgets::Widget};

    #[test]
    fn scrolled_bookmark_uses_its_absolute_index_for_highlighting() {
        let mut state = create_test_state();
        for index in 0..10 {
            state.config.bookmarks.insert(
                format!("bookmark-{index}"),
                PathBuf::from(format!("/path/{index}")),
            );
        }
        let area = Rect::new(0, 0, 60, 5);
        let mut buffer = Buffer::empty(area);

        Bookmarks::build(&state, 8, area).render(area, &mut buffer);

        assert!(buffer
            .content
            .iter()
            .any(|cell| cell.bg == Color::LightCyan));
    }
}
