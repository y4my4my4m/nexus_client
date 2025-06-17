use ratatui::{Frame, layout::Rect, widgets::Paragraph, widgets::Block, widgets::Borders};
use crate::app::App;
use crate::banner::get_styled_banner_lines;

pub fn draw_banner(f: &mut Frame, app: &App, area: Rect) {
    let banner_lines = get_styled_banner_lines(area.width, app.tick_count);
    let banner = Paragraph::new(banner_lines)
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(banner, area);
}
