pub fn build<'a>(text: &'a str) -> impl Widget + 'a {
    Paragraph::new(text).style(Style::default().fg(Color::LightCyan).bold())
}
