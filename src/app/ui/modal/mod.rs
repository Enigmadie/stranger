use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::app::{
    config::constants::ui::{COLUMN_PERCENTAGE, FOOTER_HEIGHT, HEADER_HEIGHT},
    model::miller::positions::get_position,
    state::State,
};

#[derive(Debug)]
pub enum ModalKind {
    UnderLine,
    // Custom { frame: ModalFrame },
}

trait DefaultRect {
    fn underline_default() -> Rect;
}

impl DefaultRect for Rect {
    fn underline_default() -> Rect {
        Rect {
            x: 10,
            y: 10,
            width: 10,
            height: 10,
        }
    }
}

pub struct Modal<'a> {
    state: &'a State,
    area: Rect,
}

impl<'a> Widget for Modal<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.state.modal_type {
            ModalKind::UnderLine => {
                let (x, y) = self.get_underline_pos();

                let modal_area = Rect {
                    x,
                    y,
                    height: 3 + 3,
                    width: x + 2,
                };

                Clear.render(modal_area, buf);

                let layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Length(3)])
                    .split(modal_area);

                let tab_area = layout[0]; // Верхняя часть (заголовок)
                let input_area = layout[1];

                // Заголовок с фоном
                let tab = Paragraph::new(" Rename File ")
                    .style(
                        Style::default()
                            .bg(Color::Blue)
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    )
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .style(Style::default().bg(Color::Blue)),
                    );
                tab.render(tab_area, buf);

                // Строка ввода
                let input = Paragraph::new(Text::from(self.state.input.to_string()))
                    .style(Style::default().bg(Color::Green).fg(Color::White))
                    .block(Block::default().borders(Borders::NONE));
                input.render(input_area, buf);
            }
        }
    }
}

impl<'a> Modal<'a> {
    pub fn build(state: &'a State, area: Rect) -> impl Widget + 'a {
        Modal { state, area }
    }

    pub fn get_underline_pos(&self) -> (u16, u16) {
        let body_width = self.area.width;
        let x = (body_width as f32 * (COLUMN_PERCENTAGE as f32 / 100.0)) as u16;

        let position_id = get_position(&self.state.positions_map, &self.state.current_dir) as u16;
        let item_height = 1;
        let y = HEADER_HEIGHT + (position_id * item_height) + 1;
        (x, y)
    }

    pub fn get_underline_size(&self) -> (u16, u16) {
        let body_width = self.area.width;
        let x = (body_width as f32 * (COLUMN_PERCENTAGE as f32 / 100.0)) as u16;

        let position_id = get_position(&self.state.positions_map, &self.state.current_dir) as u16;
        let item_height = 1;
        let y = HEADER_HEIGHT + (position_id * item_height);
        (x, y)
    }
}

#[cfg(test)]
mod tests {
    use crate::app::test_utils::create_test_state;

    use super::*;

    #[test]
    fn calculate_x_y() {
        let state = create_test_state();
        let area = Rect {
            x: 0,
            y: 0,
            width: 20,
            height: 20,
        };

        let modal = Modal {
            state: &state,
            area,
        };

        let (x, y) = modal.get_underline_pos();
        assert_eq!(1, y);
        assert_eq!(4, x);
    }
}
