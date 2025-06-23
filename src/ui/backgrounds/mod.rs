use ratatui::{Frame, layout::Rect};
use crate::app::App;
use crate::ui::backgrounds::cyberpunk::CyberpunkBackground;
use crate::ui::backgrounds::minimal::MinimalBackground;
use crate::ui::backgrounds::cybergrid::CyberGridBackground;
use crate::ui::backgrounds::fractalgrid::FractalGridBackground;
use crate::ui::backgrounds::geometry::GeometryBackground;
use crate::ui::backgrounds::geometry2::Geometry2Background;
use crate::ui::backgrounds::hackerglyphs::HackerGlyphsBackground;
use crate::ui::backgrounds::matrixrain::MatrixRainBackground;
use crate::ui::backgrounds::neontunnel::NeonTunnelBackground;
use crate::ui::backgrounds::plasmawave::PlasmaWaveBackground;
use crate::ui::backgrounds::pulsecircuit::PulseCircuitBackground;
use crate::ui::backgrounds::wireframeearth::WireframeEarthBackground;
use crate::ui::backgrounds::none::NoneBackground;

pub mod cyberpunk;
pub mod minimal;
pub mod cybergrid;
pub mod fractalgrid;
pub mod geometry;
pub mod geometry2;
pub mod hackerglyphs;
pub mod matrixrain;
pub mod neontunnel;
pub mod plasmawave;
pub mod pulsecircuit;
pub mod wireframeearth;
pub mod none;

pub trait Background {
    fn name(&self) -> &'static str;
    fn draw_background(&self, f: &mut Frame, app: &App, area: Rect);
}

pub struct BackgroundManager {
    backgrounds: Vec<Box<dyn Background>>,
    current_index: usize,
}

impl BackgroundManager {
    pub fn new() -> Self {
        let backgrounds: Vec<Box<dyn Background>> = vec![
            Box::new(MinimalBackground),
            Box::new(CyberpunkBackground),
            Box::new(CyberGridBackground),
            Box::new(FractalGridBackground),
            Box::new(GeometryBackground),
            Box::new(Geometry2Background),
            Box::new(HackerGlyphsBackground),
            Box::new(MatrixRainBackground),
            Box::new(NeonTunnelBackground),
            Box::new(PlasmaWaveBackground),
            Box::new(PulseCircuitBackground),
            Box::new(WireframeEarthBackground),
            Box::new(NoneBackground),
        ];
        Self {
            backgrounds,
            current_index: 0,
        }
    }
    pub fn get_current_background(&self) -> Option<&dyn Background> {
        self.backgrounds.get(self.current_index).map(|b| b.as_ref())
    }
    pub fn cycle_background(&mut self) {
        if !self.backgrounds.is_empty() {
            self.current_index = (self.current_index + 1) % self.backgrounds.len();
        }
    }
    pub fn get_background_name(&self) -> &str {
        self.get_current_background().map(|b| b.name()).unwrap_or("None")
    }
    pub fn set_background_by_name(&mut self, name: &str) {
        if let Some(idx) = self.backgrounds.iter().position(|b| b.name().eq_ignore_ascii_case(name)) {
            self.current_index = idx;
        }
    }
}
