use ratatui::{
    layout::Alignment,
    prelude::*,
    widgets::{Block, Borders, Paragraph, Widget},
};
use std::env;

trait Draw {
    fn render() -> Box<dyn Widget>;
}

pub struct Header;
pub struct Body;
pub struct Footer;

impl Draw for Header {
    fn render(&self) -> Box<dyn Widget> {
        let current_dir = env::current_dir()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|_| "Unknown directory".to_string());

        let current_dir = "";
        Box::new(
            Paragraph::new(current_dir)
                .block(Block::default().borders(Borders::BOTTOM))
                .alignment(Alignment::Left),
        )
    }
}

impl Draw for Body {
    fn render() -> Box<dyn Widget> {
        let current_dir = env::current_dir()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|_| "Unknown directory".to_string());

        let current_dir = "";
        Box::new(
            Paragraph::new(current_dir)
                .block(Block::default().borders(Borders::BOTTOM))
                .alignment(Alignment::Left),
        )
    }
}
