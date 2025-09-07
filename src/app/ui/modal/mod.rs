pub mod hint_bar;

use crate::app::{
    config::constants::ui::{COLUMN_PERCENTAGE, FIRST_COLUMN_PERCENTAGE, HEADER_HEIGHT},
    model::miller::positions::get_position,
    state::State,
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Clear, Widget},
};

#[derive(Debug)]
pub enum UnderLineModalAction {
    Add,
    Edit,
    Bookmarks,
}

#[derive(Debug)]
pub enum ModalKind {
    UnderLine { action: UnderLineModalAction },
    HintBar { mode: hint_bar::HintBarMode },
    Disabled,
    // Custom { frame: ModalFrame },
}

impl ModalKind {
    pub fn is_disabled(&self) -> bool {
        matches!(self, ModalKind::Disabled)
    }

    pub fn is_underline(&self) -> bool {
        matches!(self, ModalKind::UnderLine { .. })
    }

    pub fn is_hint_bar(&self) -> bool {
        matches!(self, ModalKind::HintBar { .. })
    }
    pub fn is_enabled(&self) -> bool {
        !self.is_disabled()
    }
}

trait _DefaultRect {
    fn underline_default() -> Rect;
}

impl _DefaultRect for Rect {
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
    state: &'a State<'a>,
    area: Rect,
}

impl<'a> Widget for Modal<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match &self.state.modal_type {
            ModalKind::UnderLine { action } => {
                let (x, y) = self.get_underline_pos();
                let (width, _) = self.get_underline_size();

                let modal_area = Rect {
                    x,
                    y,
                    height: 3,
                    width,
                };

                Clear.render(modal_area, buf);

                let backdrop = Block::default().style(Style::default());
                backdrop.render(modal_area, buf);

                let mut input = self.state.input.clone();

                let title = match action {
                    UnderLineModalAction::Add => "Add File",
                    UnderLineModalAction::Edit => "Rename File",
                    UnderLineModalAction::Bookmarks => "Add New Bookmark Name",
                };

                input.set_block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(title)
                        .style(Style::default().fg(Color::LightGreen).bold()),
                );

                input.render(modal_area, buf);
            }
            ModalKind::HintBar { mode } => hint_bar::build(area, buf, mode),
            ModalKind::Disabled => {}
        }
    }
}

impl<'a> Modal<'a> {
    pub fn build(state: &'a State, area: Rect) -> impl Widget + 'a {
        Modal { state, area }
    }

    pub fn get_underline_pos(&self) -> (u16, u16) {
        let body_width = self.area.width;
        let x = (body_width as f32 * (FIRST_COLUMN_PERCENTAGE as f32 / 100.0)) as u16;

        let position_id = get_position(&self.state.positions_map, &self.state.current_dir) as u16;
        let item_height = 1;
        let y = HEADER_HEIGHT + (position_id * item_height) + 1;
        (x, y)
    }

    pub fn get_underline_size(&self) -> (u16, u16) {
        let body_width = self.area.width;
        let width = (body_width as f32 * (COLUMN_PERCENTAGE as f32 / 100.0)) as u16;

        let position_id = get_position(&self.state.positions_map, &self.state.current_dir) as u16;
        let item_height = 1;
        let heigth = HEADER_HEIGHT + (position_id * item_height);
        (width, heigth)
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
        assert_eq!(2, y);
        assert_eq!(2, x);
    }
}
