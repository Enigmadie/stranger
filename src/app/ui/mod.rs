use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::{Frame, Rect},
    style::{Color, Style},
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
        Paragraph::new(state.current_dir.display().to_string())
            .block(Block::default().borders(Borders::BOTTOM))
            .alignment(Alignment::Left)
    }
}

struct ColumnsWidget<'a> {
    lists: Vec<List<'a>>,
}

impl<'a> ColumnsWidget<'a> {
    fn new(lists: Vec<List<'a>>) -> Self {
        ColumnsWidget { lists }
    }
}

impl<'a> Widget for ColumnsWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let constraints: Vec<Constraint> = self
            .lists
            .iter()
            .enumerate()
            .map(|(i, _)| {
                if i >= 2 {
                    Constraint::Percentage(60)
                } else {
                    Constraint::Percentage(20)
                }
            })
            .collect();
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        for (i, list) in self.lists.into_iter().enumerate() {
            list.render(layout[i], buf);
        }
    }
}

impl Body {
    fn render<'a>(state: &'a State, _area: Rect) -> impl Widget + 'a {
        let lists: Vec<List> = state
            .files
            .iter()
            .enumerate()
            .map(|(col_id, dir)| {
                let is_parent_column = col_id == 0;
                let is_current_column = col_id == 1;
                let is_child_column = col_id >= 2;

                let list_items: Vec<ListItem> = if is_parent_column && dir.is_empty() {
                    // if parent dir is empty
                    vec![]
                } else if (is_current_column || is_child_column) && dir.is_empty() {
                    // is current or child dir are empty
                    vec![ListItem::new("empty")]
                } else {
                    dir.iter()
                        .enumerate()
                        .map(|(row_id, file)| {
                            let list_item = ListItem::new(file.name.as_str());
                            if is_current_column && row_id == state.position_id {
                                list_item.style(Style::default().bg(Color::White).fg(Color::Black))
                            } else {
                                list_item
                            }
                        })
                        .collect()
                };
                List::new(list_items).block(Block::default())
            })
            .collect();

        ColumnsWidget::new(lists)
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
    use std::path::PathBuf;

    use crate::app::model::file_entry::{FileEntry, FileVariant};

    use super::*;

    fn create_test_state() -> State {
        State {
            current_dir: PathBuf::from("/src/ui/tests"),
            exit: false,
            files: [
                vec![],
                vec![FileEntry {
                    name: "file1".into(),
                    variant: FileVariant::File,
                }],
                vec![],
            ],
            needs_redraw: false,
            position_id: 0,
        }
    }

    #[test]
    fn header_path() {
        let state = create_test_state();
        let area = Rect::new(0, 1, 200, 1);
        let header = Header::render(&state, area);

        assert!(format!("{:?}", header).contains(&state.current_dir.display().to_string()));
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
