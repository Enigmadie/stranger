use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem},
};
use std::io::{self, Stdout};

use super::draw;
use super::state::State;

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
    show_popup: bool,
}

impl App {
    pub fn new() -> io::Result<Self> {
        Ok(App {
            state: state::new(),
        })
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|f| self.draw(f))?;
            let _ = self.handle_events();
        }
        Ok(())
    }

    pub fn draw(&self, frame: &mut Frame<'_>) {
        let area = frame.size();

        let items = vec![
            ListItem::new("file1"),
            ListItem::new("file2"),
            ListItem::new("file3"),
        ];

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ])
            .split(area);

        let block = Block::default().borders(Borders::ALL);

        let list = List::new(items).block(block);

        frame.render_widget(list, layout[1]);
    }

    pub fn handle_events(&mut self) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') {
                self.exit = true;
            }
        }
        Ok(())
    }
}
