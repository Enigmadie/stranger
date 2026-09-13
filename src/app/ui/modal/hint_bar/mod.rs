use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Clear, Row, Table, Widget},
};

use crate::app::utils::i18n::Lang;

#[derive(Debug)]
pub enum HintBarMode {
    Bookmarks,
    Delete,
    Exit,
}

pub fn build(area: Rect, buf: &mut Buffer, mode: &HintBarMode) {
    let height = 10.min(area.height);
    if height == 0 || area.width == 0 {
        return;
    }

    let modal_area = Rect {
        x: area.x,
        y: area.y.saturating_add(area.height.saturating_sub(height)),
        height,
        width: area.width,
    };

    Clear.render(modal_area, buf);

    let backdrop = Block::default().style(Style::default());
    backdrop.render(modal_area, buf);

    let list = match mode {
        HintBarMode::Bookmarks => vec![
            ("b", Lang::en("hint_bookmark_list")),
            ("a", Lang::en("hint_add_bookmark")),
        ],
        HintBarMode::Delete => vec![
            ("d", Lang::en("hint_cut_files")),
            ("D", Lang::en("hint_delete_to_trash")),
            ("x", Lang::en("hint_delete_permanently")),
        ],
        HintBarMode::Exit => vec![
            ("Z", Lang::en("hint_exit_current_directory")),
            ("Q", Lang::en("hint_exit_initial_directory")),
        ],
    };

    let rows: Vec<Row> = list
        .iter()
        .map(|(key, action)| {
            Row::new(vec![
                Cell::from(format!(" {key}")).style(Style::default().fg(Color::Yellow)),
                Cell::from((*action).to_string()),
            ])
        })
        .collect();

    Table::new(rows, [Constraint::Length(12), Constraint::Min(10)])
        .header(
            Row::new(vec![
                Lang::en("hint_column_key"),
                Lang::en("hint_column_action"),
            ])
            .style(Style::default().fg(Color::Cyan)),
        )
        .block(Block::default().borders(Borders::TOP))
        .style(Style::default().fg(Color::White))
        .render(modal_area, buf);
}
