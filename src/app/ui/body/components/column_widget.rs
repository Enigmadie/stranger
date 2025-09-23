use std::rc::Rc;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Clear, List, Paragraph, Widget},
};

#[derive(Clone)]
pub enum ColumnWidget<'a> {
    List(List<'a>),
    Paragraph(Paragraph<'a>),
}

impl<'a> Widget for ColumnWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self {
            ColumnWidget::List(list) => list.render(area, buf),
            ColumnWidget::Paragraph(paragraph) => paragraph.render(area, buf),
        }
    }
}

pub struct ColumnsWidget<'a> {
    widgets: Vec<ColumnWidget<'a>>,
    layout: Rc<[Rect]>,
}

impl<'a> ColumnsWidget<'a> {
    pub fn new(widgets: Vec<ColumnWidget<'a>>, layout: Rc<[Rect]>) -> Self {
        ColumnsWidget { widgets, layout }
    }

    pub fn layout(&self) -> &Rc<[Rect]> {
        &self.layout
    }
}

impl<'a> Widget for ColumnsWidget<'a> {
    fn render(self, _area: Rect, buf: &mut Buffer) {
        for (i, widget) in self.widgets.into_iter().enumerate() {
            let col_area = self.layout[i];

            Clear.render(col_area, buf);

            widget.render(col_area, buf);
        }
    }
}
