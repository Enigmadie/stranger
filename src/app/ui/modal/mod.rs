use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::app::{
    config::constants::ui::{COLUMN_PERCENTAGE, FOOTER_HEIGHT, HEADER_HEIGHT},
    state::State,
};

#[derive(Debug)]
pub enum ModalKind {
    UnderLine,
    // Custom { frame: ModalFrame },
}

#[derive(Debug)]
pub struct ModalRect {
    x: u16,
    y: u16,
    width: u16,
    heigth: u16,
}

impl ModalRect {
    pub fn default() -> Self {
        ModalRect {
            x: 400,
            y: 200,
            width: 400,
            heigth: 200,
        }
    }
}

pub struct Modal<'a> {
    state: &'a State,
    area: Rect,
}

impl<'a> Widget for Modal<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let backdrop = Block::default().style(Style::default().bg(Color::Black));
        backdrop.render(area, buf);

        match self.state.modal_type {
            ModalKind::UnderLine => {
                let modal = Paragraph::new(Text::from(self.state.input.clone())).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Rename File")
                        .style(Style::default().bg(Color::Black).fg(Color::White)),
                );
                modal.render(area, buf);
            }
        }
    }
}

impl<'a> Modal<'a> {
    pub fn build(state: &'a State, area: Rect) -> impl Widget + 'a {
        Modal { state, area }
    }

    pub fn get_underline_y(&self, position_id: u16) -> u16 {
        let base_y = HEADER_HEIGHT;
        let body_height = self
            .area
            .height
            .saturating_sub(HEADER_HEIGHT + FOOTER_HEIGHT);
        let column_y = base_y + (body_height as f32 * (COLUMN_PERCENTAGE as f32 / 100.0)) as u16;
        let item_height = 1;
        column_y + (position_id * item_height) + item_height
    }
}
