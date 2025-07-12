use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
};

use crate::app::state::State;

#[derive(Debug)]
pub enum ModalKind {
    Default,
}

pub struct ModalFrame {
    x: u16,
    y: u16,
    width: u16,
    heigth: u16,
}
pub struct Modal {
    kind: ModalKind,
    position: ModalFrame,
}

impl Widget for Modal {
    fn render(self, area: Rect, buf: &mut Buffer) -> impl Widget {
        let modal = Block::default()
            .borders(Borders::ALL)
            .title("Modal")
            .style(Style::default().bg(Color::Black).fg(Color::White));
        modal
    }
}

impl Modal {
    pub fn build(state: &State, _area: Rect) -> impl Widget {
        Modal {}
    }
}
