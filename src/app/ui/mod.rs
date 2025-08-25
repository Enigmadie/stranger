pub mod body;
pub mod modal;
pub mod preview;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::{Frame, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::app::{
    config::constants::ui::{FOOTER_HEIGHT, HEADER_HEIGHT},
    model::{file::get_current_file, miller::entries::FileVariant, notification::Notification},
    state::State,
    ui::{body::Body, modal::Modal},
    utils::{format_bytes, fs::whoami_info},
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

    frame.render_widget(header, layout[0]);
    frame.render_widget(body, layout[1]);
    frame.render_widget(footer, layout[2]);

    if state.show_popup {
        let modal = Modal::build(state, area);
        frame.render_widget(modal, area);
    }
}

pub struct Header;
pub struct Footer;

impl Header {
    fn build<'a>(state: &'a State, _area: Rect) -> impl Widget + 'a {
        let dir = state.current_dir.display().to_string();
        let file = get_current_file(&state.positions_map, &state.current_dir, &state.files[1])
            .map(|e| e.name.to_owned())
            .unwrap_or(String::from(""));

        let user_info = whoami_info().unwrap_or_else(|_| String::from("unknown@localhost"));

        let text = Line::from(vec![
            Span::styled(user_info, Style::default().fg(Color::Green).bold()),
            Span::raw(" "),
            Span::styled(format!("{}/", dir), Style::default().fg(Color::Blue).bold()),
            Span::raw(file).bold(),
        ]);

        Paragraph::new(text)
            .block(Block::default())
            .alignment(Alignment::Left)
    }
}

impl Footer {
    fn build<'a>(state: &'a State, _area: Rect) -> impl Widget + 'a {
        if let Some(notification) = &state.notification {
            let (msg, color) = match notification {
                Notification::Info { msg } => (msg, Color::White),
                Notification::Success { msg } => (msg, Color::LightGreen),
                Notification::Warn { msg } => (msg, Color::LightYellow),
                Notification::Error { msg } => (msg, Color::LightRed),
            };

            Paragraph::new(msg.as_ref())
                .style(Style::default().fg(color))
                .block(Block::default().borders(Borders::NONE))
                .alignment(Alignment::Left)
        } else {
            let (permissions, size, last_modified): (String, String, String) =
                get_current_file(&state.positions_map, &state.current_dir, &state.files[1])
                    .map(|file| match &file.variant {
                        FileVariant::Directory {
                            permissions,
                            len,
                            last_modified,
                        } => (
                            permissions.clone().unwrap_or_default(),
                            len.unwrap_or_default().to_string(),
                            last_modified.clone().unwrap_or_default(),
                        ),
                        FileVariant::File {
                            permissions,
                            size,
                            last_modified,
                        } => (
                            permissions.clone().unwrap_or_default(),
                            size.map(format_bytes).unwrap_or_default(),
                            last_modified.clone().unwrap_or_default(),
                        ),
                    })
                    .unwrap_or_default();

            let text = Line::from(vec![
                Span::styled(permissions, Style::default().fg(Color::LightBlue).bold()),
                Span::raw(" "),
                Span::styled(last_modified, Style::default().fg(Color::White).bold()),
                Span::raw(" "),
                Span::styled(size, Style::default().fg(Color::LightGreen).bold()),
            ]);

            Paragraph::new(text)
                .block(Block::default())
                .alignment(Alignment::Left)
        }
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
