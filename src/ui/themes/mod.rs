use ratatui::style::{Style, Color};
use ratatui::layout::{Constraint, Rect};

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

pub struct ThemeMainMenuLayout {
    pub constraints: Vec<Constraint>,
    pub show_top_banner: bool,
    pub show_status: bool,
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
    /// Draw the top banner (or nothing for minimal themes)
    fn draw_top_banner(&self, f: &mut ratatui::Frame, app: &crate::app::App, area: ratatui::layout::Rect);
    /// Draw the bottom banner (or nothing for minimal themes)
    fn draw_bottom_banner(&self, f: &mut ratatui::Frame, app: &crate::app::App, area: ratatui::layout::Rect);
    /// Draw the main menu (fancy or minimal)
    fn draw_main_menu(&self, f: &mut ratatui::Frame, main_menu_state: &mut ratatui::widgets::ListState, tick: u64, area: ratatui::layout::Rect);
    /// Draw the settings menu (fancy or minimal)
    fn draw_settings_menu(&self, f: &mut ratatui::Frame, settings_list_state: &mut ratatui::widgets::ListState, tick: u64, area: ratatui::layout::Rect);
    /// Draw floating UI elements (corners, tick counter, etc)
    fn draw_floating_elements(&self, f: &mut ratatui::Frame, app: &crate::app::App, area: ratatui::layout::Rect);
    /// Get the layout for the main menu
    fn main_menu_layout(&self, area: Rect) -> ThemeMainMenuLayout;
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
    pub fn set_theme_by_name(&mut self, name: &str) {
        if let Some(idx) = self.themes.iter().position(|t| t.name().eq_ignore_ascii_case(name)) {
            self.current_index = idx;
        }
    }
}