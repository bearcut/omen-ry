use std::time::Duration;
use crate::color::palettes::{get_palette, Palette};
use crate::color::Color;
use super::Animation;

#[derive(Debug, Clone)]
pub struct WaveAnimation {
    palette: Palette,
    speed: f32,       // Cycles per second
    wavelength: f32,  // Number of zones per cycle
    reverse: bool,    // Flow right-to-left if true
    elapsed: f32,     // Running timestamp in seconds
}

impl WaveAnimation {
    pub fn new(palette_name: &str, speed: f32, reverse: bool) -> Self {
        Self {
            palette: get_palette(palette_name),
            speed: speed.max(0.01),
            wavelength: 4.0,
            reverse,
            elapsed: 0.0,
        }
    }

    pub fn set_palette(&mut self, palette_name: &str) {
        self.palette = get_palette(palette_name);
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed.max(0.01);
    }

    pub fn set_reverse(&mut self, reverse: bool) {
        self.reverse = reverse;
    }
}

impl Default for WaveAnimation {
    fn default() -> Self {
        Self::new("cyberpunk", 0.5, false)
    }
}

impl Animation for WaveAnimation {
    fn name(&self) -> &'static str {
        "wave"
    }

    fn is_animating(&self) -> bool {
        true
    }

    fn tick(&mut self, dt: Duration) -> [Color; 4] {
        self.elapsed += dt.as_secs_f32();

        let mut colors = [Color::BLACK; 4];
        for (zone, color) in colors.iter_mut().enumerate() {
            let spatial_idx = if self.reverse { 3 - zone } else { zone };
            let spatial_offset = spatial_idx as f32 / self.wavelength;
            let phase = (self.elapsed * self.speed + spatial_offset).rem_euclid(1.0);

            *color = self.palette.sample(phase);
        }

        colors
    }
}
