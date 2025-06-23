use ratatui::{Frame, layout::Rect, style::{Style, Color}, widgets::Paragraph, text::Span};
use ratatui::style::Modifier;
use crate::app::App;

mod cyberpunk;
mod minimal;

pub use cyberpunk::CyberpunkTheme;
pub use minimal::MinimalTheme;

#[derive(Clone)]
pub struct ThemeColors {
    pub primary: Color,
    pub secondary: Color,
    pub background: Color,
    pub text: Color,
    pub selected_bg: Color,
    pub selected_fg: Color,
}

#[derive(Clone)]
pub struct AccentColors {
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
}

/// Trait for defining UI color themes (palette, widget styles, etc.)
pub trait Theme {
    /// Name of the theme
    fn name(&self) -> &'static str;
    /// Main color palette
    fn colors(&self) -> ThemeColors;
    /// Accent colors
    fn accents(&self) -> AccentColors;
    /// Border style for blocks
    fn border_color(&self, tick: u64) -> Color;
    /// Style for selected items
    fn selected_style(&self) -> Style;
    /// Style for normal text
    fn text_style(&self) -> Style;
}

/// Theme manager for cycling through available UI color themes
pub struct ThemeManager {
    themes: Vec<Box<dyn Theme>>,
    current_index: usize,
}

impl ThemeManager {
    pub fn new() -> Self {
        let themes: Vec<Box<dyn Theme>> = vec![
            Box::new(CyberpunkTheme),
            Box::new(MinimalTheme),
        ];
        Self { themes, current_index: 0 }
    }
    pub fn get_current_theme(&self) -> &dyn Theme {
        self.themes[self.current_index].as_ref()
    }
    pub fn cycle_theme(&mut self) {
        self.current_index = (self.current_index + 1) % self.themes.len();
    }
    pub fn get_theme_name(&self) -> &str {
        self.get_current_theme().name()
    }
}