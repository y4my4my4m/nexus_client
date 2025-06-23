use ratatui::{Frame, layout::Rect, style::{Style, Color}, widgets::Paragraph, text::Span};
use crate::app::App;

pub mod cyberpunk;
pub mod minimal;
pub mod geometry;

pub use cyberpunk::CyberpunkTheme;
pub use minimal::MinimalTheme;
pub use geometry::GeometryTheme;

/// Trait for defining UI themes
pub trait Theme {
    /// Get the name of this theme
    fn name(&self) -> &'static str;
    
    /// Draw the animated background for this theme
    fn draw_background(&self, f: &mut Frame, app: &App, area: Rect);
    
    /// Get the primary color scheme for this theme
    fn get_primary_colors(&self) -> ThemeColors;
    
    /// Get border colors that cycle with animation
    fn get_border_colors(&self, tick: u64) -> Color;
    
    /// Get accent colors for special elements
    fn get_accent_colors(&self) -> AccentColors;
}

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

/// Theme manager for cycling through available themes
pub struct ThemeManager {
    themes: Vec<Box<dyn Theme>>,
    current_index: usize,
}

impl ThemeManager {
    pub fn new() -> Self {
        let themes: Vec<Box<dyn Theme>> = vec![
            Box::new(MinimalTheme),
            Box::new(CyberpunkTheme),
            Box::new(GeometryTheme),
        ];
        
        Self {
            themes,
            current_index: 0,
        }
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