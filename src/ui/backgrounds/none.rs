use ratatui::{Frame, layout::Rect, style::{Style, Color}, widgets::Paragraph};
use crate::app::App;
use crate::ui::backgrounds::Background;

pub struct NoneBackground;

impl Background for NoneBackground {
    fn name(&self) -> &'static str {
        "None"
    }
    
    fn draw_background(&self, f: &mut Frame, app: &App, area: Rect) {
    }
}