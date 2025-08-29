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
        Row,
    },
};

pub struct Bookmarks;

impl Bookmarks {
    pub fn build<'a>(state: &'a State, area: Rect) -> impl Widget + 'a {
        let (names, paths): (Vec<&String>, Vec<&PathBuf>) = state.config.bookmarks.iter().unzip();

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);

        let cursor = 0;
        const TARGET_POSITION_DOWN: usize = 6;

        let visible_height = layout[0].height.saturating_sub(2) as usize;

        let offset = if names.len() > visible_height {
            let max_possible_offset = names.len().saturating_sub(visible_height);
            let upper_bound = visible_height - TARGET_POSITION_DOWN; // 30 - 6 = 24;
            if cursor >= upper_bound {
                (cursor - upper_bound).min(max_possible_offset)
            } else {
                0
            }
        } else {
            0
        };

        let list_names: Vec<ListItem> = names
            .iter()
            .skip(offset)
            .take(visible_height)
            .enumerate()
            .map(|(row_id, file)| Row::bookmarks_build(row_id, file.to_string(), true, cursor))
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
            .map(|(row_id, path)| Row::bookmarks_build(row_id, path.to_string(), false, cursor))
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
