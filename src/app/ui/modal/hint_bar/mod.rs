use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Clear, Row, Table, Widget},
};

use crate::app::ui::modal::HintBarMode;

pub fn build(area: Rect, buf: &mut Buffer, mode: &HintBarMode) {
    let height = 10;

    let modal_area = Rect {
        x: 0,
        y: area.height.saturating_sub(height),
        height,
        width: area.width,
    };

    Clear.render(modal_area, buf);

    let backdrop = Block::default().style(Style::default());
    backdrop.render(modal_area, buf);

    let list = match mode {
        HintBarMode::Bookmarks => vec![
            ("a", "Add Bookmark"),
            ("d", "Delete Bookmark"),
            ("l/Enter", "Go to Bookmark"),
            ("q", "Close"),
        ],
    };

    let rows: Vec<Row> = list
        .iter()
        .map(|(key, action)| {
            Row::new(vec![
                Cell::from(key.to_string()).style(Style::default().fg(Color::Yellow)),
                Cell::from((*action).to_string()),
            ])
        })
        .collect();

    Table::new(rows, [Constraint::Length(12), Constraint::Min(10)])
        .header(Row::new(vec!["Key", "Action"]).style(Style::default().fg(Color::Cyan)))
        .block(Block::default().borders(Borders::TOP))
        .style(Style::default().fg(Color::White))
        .render(modal_area, buf);
}
