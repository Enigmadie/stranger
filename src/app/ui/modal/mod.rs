pub mod hint_bar;

use crate::app::{
    config::constants::ui::{
        COLUMN_PERCENTAGE, FIRST_COLUMN_PERCENTAGE, FOOTER_HEIGHT, HEADER_HEIGHT,
    },
    model::miller::positions::get_position,
    state::State,
    ui::body::viewport_offset,
    utils::i18n::Lang,
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Clear, Widget},
};

#[derive(Debug, Clone, Copy)]
pub enum UnderLineModalAction {
    Add,
    Edit,
    Bookmarks,
}

#[derive(Debug)]
pub enum ModalKind {
    UnderLine { action: UnderLineModalAction },
    HintBar { mode: hint_bar::HintBarMode },
    BottomLine,
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

    pub fn is_bottom_line(&self) -> bool {
        matches!(self, ModalKind::BottomLine)
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

                let requested_area = Rect {
                    x,
                    y,
                    height: 3,
                    width,
                };
                let Some(modal_area) = clipped_rect(requested_area, area) else {
                    return;
                };

                Clear.render(modal_area, buf);

                let backdrop = Block::default().style(Style::default());
                backdrop.render(modal_area, buf);

                let mut input = self.state.input.clone();

                let title = match action {
                    UnderLineModalAction::Add => Lang::en("modal_add_file_title"),
                    UnderLineModalAction::Edit => Lang::en("modal_rename_file_title"),
                    UnderLineModalAction::Bookmarks => Lang::en("modal_add_bookmark_title"),
                };

                input.set_block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(title)
                        .style(Style::default().fg(Color::LightGreen).bold()),
                );
                input.set_cursor_line_style(Style::default());

                input.render(modal_area, buf);
            }
            ModalKind::HintBar { mode } => hint_bar::build(area, buf, mode),
            ModalKind::BottomLine => {
                let requested_area = Rect {
                    x: area.x,
                    y: area.y.saturating_add(area.height.saturating_sub(1)),
                    width: area.width,
                    height: 1,
                };
                let Some(modal_area) = clipped_rect(requested_area, area) else {
                    return;
                };

                Clear.render(modal_area, buf);

                let backdrop = Block::default().style(Style::default());
                backdrop.render(modal_area, buf);

                let mut input = self.state.input.clone();

                input.set_block(Block::default().style(Style::default().fg(Color::White).bold()));
                input.set_cursor_line_style(Style::default());

                input.render(modal_area, buf);
            }
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
        let x = self
            .area
            .x
            .saturating_add((body_width as f32 * (FIRST_COLUMN_PERCENTAGE as f32 / 100.0)) as u16);

        let position_id = get_position(&self.state.positions_map, &self.state.current_dir);
        let body_height = self
            .area
            .height
            .saturating_sub(HEADER_HEIGHT.saturating_add(FOOTER_HEIGHT));
        let visible_height = body_height.saturating_sub(2) as usize;
        let offset = viewport_offset(self.state.files[1].len(), visible_height, position_id);
        let visible_position = position_id.saturating_sub(offset) as u16;
        let y = self
            .area
            .y
            .saturating_add(HEADER_HEIGHT)
            .saturating_add(visible_position)
            .saturating_add(1);
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

fn clipped_rect(rect: Rect, area: Rect) -> Option<Rect> {
    let x = rect.x.max(area.x);
    let y = rect.y.max(area.y);
    let right = rect.right().min(area.right());
    let bottom = rect.bottom().min(area.bottom());
    (right > x && bottom > y).then(|| Rect::new(x, y, right - x, bottom - y))
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

    #[test]
    fn rectangles_are_clipped_to_the_render_area() {
        let area = Rect::new(5, 5, 10, 4);

        assert_eq!(
            clipped_rect(Rect::new(3, 7, 20, 5), area),
            Some(Rect::new(5, 7, 10, 2))
        );
        assert_eq!(clipped_rect(Rect::new(0, 0, 2, 2), area), None);
    }
}
