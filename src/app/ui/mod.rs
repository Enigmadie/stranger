pub mod body;
pub mod modal;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::{Frame, Rect},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::app::{
    config::constants::ui::{FOOTER_HEIGHT, HEADER_HEIGHT},
    state::State,
    ui::{body::Body, modal::Modal},
};

pub fn render(state: &State, frame: &mut Frame<'_>) {
    let area = frame.area();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(HEADER_HEIGHT),
            Constraint::Min(0),
            Constraint::Length(FOOTER_HEIGHT),
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
        frame.render_widget(modal, area);
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
    use ratatui::buffer::Buffer;

    use crate::app::test_utils::create_test_state;

    use super::*;

    #[test]
    fn header_path() {
        let state = create_test_state();
        let area = Rect {
            x: 1,
            y: 1,
            width: 150,
            height: 150,
        };
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
        let area = Rect {
            x: 0,
            y: 0,
            width: 50,
            height: 3,
        };
        let footer = Footer::build(&state, area);

        let mut buffer = Buffer::empty(area);
        footer.render(area, &mut buffer);

        let text: String = buffer
            .content
            .iter()
            .filter_map(|cell| cell.symbol().chars().next())
            .collect();

        assert!(text.contains("Press q to quit"));
    }
}
