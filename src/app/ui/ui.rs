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

pub trait Draw {
    fn render(state: &State, area: Rect) -> impl Widget;
}

pub struct Header;
pub struct Body;
pub struct Footer;

impl Draw for Header {
    fn render(state: &State, _area: Rect) -> impl Widget {
        Paragraph::new(state.current_dir.as_str())
            .block(Block::default().borders(Borders::BOTTOM))
            .alignment(Alignment::Left)
    }
}

impl Draw for Body {
    fn render(state: &State, _area: Rect) -> impl Widget {
        let list: Vec<ListItem> = state
            .files
            .iter()
            .map(|e| ListItem::new(e.as_str()))
            .collect();

        List::new(list).block(Block::default())
    }
}

impl Draw for Footer {
    fn render(_state: &State, _area: Rect) -> impl Widget {
        Paragraph::new("Press q to quit")
            .block(Block::default().borders(Borders::NONE))
            .alignment(Alignment::Center)
    }
}
