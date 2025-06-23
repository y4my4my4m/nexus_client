use ratatui::{Frame, layout::Rect, style::{Style, Color}, widgets::Paragraph, text::Span};
use crate::app::App;

pub mod cyberpunk;
pub mod minimal;
pub mod geometry;
pub mod geometry2;
pub mod cybergrid;
pub mod plasmawave;
pub mod matrixrain;
pub mod neontunnel;
pub mod fractalgrid;
pub mod pulsecircuit;
pub mod hackerglyphs;
pub mod wireframeearth;

pub use cyberpunk::CyberpunkTheme;
pub use minimal::MinimalTheme;
pub use geometry::GeometryTheme;
pub use geometry2::Geometry2Theme;
pub use cybergrid::CyberGridTheme;
pub use plasmawave::PlasmaWaveTheme;
pub use matrixrain::MatrixRainTheme;
pub use neontunnel::NeonTunnelTheme;
pub use fractalgrid::FractalGridTheme;
pub use pulsecircuit::PulseCircuitTheme;
pub use hackerglyphs::HackerGlyphsTheme;
pub use wireframeearth::WireframeEarthTheme;

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
            Box::new(Geometry2Theme),
            Box::new(CyberGridTheme),
            Box::new(PlasmaWaveTheme),
            Box::new(MatrixRainTheme),
            Box::new(NeonTunnelTheme),
            Box::new(FractalGridTheme),
            Box::new(PulseCircuitTheme),
            Box::new(HackerGlyphsTheme),
            Box::new(WireframeEarthTheme),
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