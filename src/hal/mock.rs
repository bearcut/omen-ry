use std::io::{self, Write};
use crate::color::Color;
use super::ZoneBackend;

pub struct MockBackend {
    cached_colors: [Color; 4],
    cached_brightness: u8,
    interactive: bool,
}

impl MockBackend {
    pub fn new(interactive: bool) -> Self {
        Self {
            cached_colors: [Color::BLACK; 4],
            cached_brightness: 100,
            interactive,
        }
    }

    pub fn render_preview(colors: &[Color; 4], brightness: u8) -> String {
        let scaled: Vec<Color> = colors.iter().map(|c| c.scale_brightness(brightness)).collect();
        format!(
            "[Z0: \x1b[48;2;{};{};{}m  \x1b[0m {}] [Z1: \x1b[48;2;{};{};{}m  \x1b[0m {}] [Z2: \x1b[48;2;{};{};{}m  \x1b[0m {}] [Z3: \x1b[48;2;{};{};{}m  \x1b[0m {}]",
            scaled[0].r, scaled[0].g, scaled[0].b, scaled[0].to_hex_hash(),
            scaled[1].r, scaled[1].g, scaled[1].b, scaled[1].to_hex_hash(),
            scaled[2].r, scaled[2].g, scaled[2].b, scaled[2].to_hex_hash(),
            scaled[3].r, scaled[3].g, scaled[3].b, scaled[3].to_hex_hash(),
        )
    }
}

impl ZoneBackend for MockBackend {
    fn set_zones(&mut self, colors: &[Color; 4]) -> io::Result<()> {
        if self.cached_colors != *colors {
            self.cached_colors = *colors;
            if self.interactive {
                print!("\r{}", Self::render_preview(&self.cached_colors, self.cached_brightness));
                io::stdout().flush()?;
            }
        }
        Ok(())
    }

    fn set_brightness(&mut self, brightness: u8) -> io::Result<()> {
        self.cached_brightness = brightness.min(100);
        if self.interactive {
            print!("\r{}", Self::render_preview(&self.cached_colors, self.cached_brightness));
            io::stdout().flush()?;
        }
        Ok(())
    }

    fn backend_name(&self) -> &'static str {
        "Mock / Terminal Visualizer"
    }
}
