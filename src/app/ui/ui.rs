use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::{Frame, Rect},
    widgets::{Block, Borders, List, ListItem, Paragraph, Widget},
};

use crate::app::state::State;

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

    let header = Header::render(state, area);
    let body = Body::render(state, area);
    let footer = Footer::render(state, area);

    frame.render_widget(header, layout[0]);
    frame.render_widget(body, layout[1]);
    frame.render_widget(footer, layout[2]);
}

pub struct Header;
pub struct Body;
pub struct Footer;

impl Header {
    fn render(state: &State, _area: Rect) -> Paragraph {
        Paragraph::new(state.current_dir.as_str())
            .block(Block::default().borders(Borders::BOTTOM))
            .alignment(Alignment::Left)
    }
}

impl Body {
    fn render(state: &State, _area: Rect) -> List {
        let list: Vec<ListItem> = state
            .files
            .iter()
            .map(|e| ListItem::new(e.as_str()))
            .collect();

        List::new(list).block(Block::default())
    }
}

impl Footer {
    fn render(_state: &State, _area: Rect) -> Paragraph {
        Paragraph::new("Press q to quit")
            .block(Block::default().borders(Borders::NONE))
            .alignment(Alignment::Center)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_state() -> State {
        State {
            current_dir: "/src/ui/tests".into(),
            exit: false,
            files: vec!["file1".into(), "file2".into()],
            needs_redraw: false,
        }
    }

    #[test]
    fn header_path() {
        let state = create_test_state();
        let area = Rect::new(0, 1, 200, 1);
        let header = Header::render(&state, area);

        assert!(format!("{:?}", header).contains(state.current_dir.as_str()));
    }

    #[test]
    fn footer_path() {
        let state = create_test_state();
        let area = Rect::new(0, 1, 200, 1);
        let footer = Footer::render(&state, area);

        println!("{:?}", footer);
        assert!(format!("{:?}", footer).contains("Press q to quit"));
    }
}
