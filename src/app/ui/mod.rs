pub mod body;
pub mod modal;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::{Frame, Rect},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::app::{
    state::State,
    ui::{body::Body, modal::Modal},
};

pub fn render(state: &State, frame: &mut Frame<'_>) {
    let area = frame.size();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let header = Header::build(state, layout[0]);
    let body = Body::build(state, layout[1]);
    let footer = Footer::build(state, layout[2]);
    // let modal = Modal

    frame.render_widget(header, layout[0]);
    frame.render_widget(body, layout[1]);
    frame.render_widget(footer, layout[2]);

    if state.show_popup {
        let modal = Modal::build(state, area);
    }
    // frame.render_widget(, area);
}

pub struct Header;
pub struct Footer;

impl Header {
    fn build<'a>(state: &'a State, _area: Rect) -> impl Widget + 'a {
        Paragraph::new(state.current_dir.display().to_string())
            .block(Block::default().borders(Borders::BOTTOM))
            .alignment(Alignment::Left)
    }
}

impl Footer {
    fn build<'a>(_state: &'a State, _area: Rect) -> impl Widget + 'a {
        Paragraph::new("Press q to quit")
            .block(Block::default().borders(Borders::NONE))
            .alignment(Alignment::Center)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::PathBuf};

    use ratatui::buffer::Buffer;

    use crate::app::{
        model::file_entry::{FileEntry, FileVariant},
        state::Mode,
    };

    use super::*;

    fn create_test_state() -> State {
        let mut positions_map: HashMap<PathBuf, usize> = HashMap::new();
        let current_dir = PathBuf::from("/src/ui/tests");

        positions_map.insert(current_dir.clone(), 0);

        State {
            current_dir,
            exit: false,
            files: [
                vec![],
                vec![FileEntry {
                    name: "file1".into(),
                    variant: FileVariant::File,
                }],
                vec![],
            ],
            dirs: [
                Some(PathBuf::from("src/ui")),
                Some(PathBuf::from("src/ui")),
                Some(PathBuf::from("src/ui")),
            ],
            mode: Mode::Normal,
            show_popup: false,
            needs_redraw: false,
            positions_map,
        }
    }

    #[test]
    fn header_path() {
        let state = create_test_state();
        let area = Rect::new(0, 1, 200, 1);
        let header = Header::build(&state, area);

        let mut buffer = Buffer::empty(area);
        header.render(area, &mut buffer);

        let text: String = buffer
            .content
            .iter()
            .filter_map(|cell| cell.symbol().chars().next())
            .collect();

        assert!(text.contains(&state.current_dir.display().to_string()));
    }

    #[test]
    fn footer_path() {
        let state = create_test_state();
        let area = Rect::new(0, 1, 200, 1);
        let footer = Footer::build(&state, area);

        let mut buffer = Buffer::empty(area);
        footer.render(area, &mut buffer);

        let text: String = buffer
            .content
            .iter()
            .filter_map(|cell| cell.symbol().chars().next())
            .collect();

        assert!(text.contains(&state.current_dir.display().to_string()));
    }
}
