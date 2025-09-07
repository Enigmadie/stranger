use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Clear, Row, Table, Widget},
};

#[derive(Debug)]
pub enum HintBarMode {
    Bookmarks,
    Delete,
}

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
        HintBarMode::Bookmarks => vec![("b", "Bookmark List"), ("a", "Add Bookmark")],
        HintBarMode::Delete => vec![
            ("d", "Cut Files"),
            (
                "D",
                "Delete Files To Trash (On macOS, if prompted, please grant file acces. If not granted, files will be deleted permanently.)",
            ),
            ("x", "Delete Files Permanently"),
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
