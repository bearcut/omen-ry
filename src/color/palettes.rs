use super::{Color, Hsv};

#[derive(Debug, Clone)]
pub struct Palette {
    pub name: &'static str,
    pub colors: Vec<Color>,
}

impl Palette {
    pub fn new(name: &'static str, colors: Vec<Color>) -> Self {
        assert!(!colors.is_empty(), "Palette cannot be empty");
        Self { name, colors }
    }

    /// Samples a color along the palette cyclically for t in [0.0, 1.0]
    pub fn sample(&self, t: f32) -> Color {
        let count = self.colors.len();
        if count == 1 {
            return self.colors[0];
        }

        let normalized = t.rem_euclid(1.0);
        let scaled = normalized * count as f32;
        let idx = scaled.floor() as usize % count;
        let next_idx = (idx + 1) % count;
        let frac = scaled.fract();

        self.colors[idx].lerp(&self.colors[next_idx], frac)
    }
}

pub fn get_palette(name: &str) -> Palette {
    match name.to_lowercase().as_str() {
        "cyberpunk" => Palette::new(
            "Cyberpunk",
            vec![
                Color::new(0, 240, 255),   // Neon Cyan
                Color::new(128, 0, 255),   // Violet
                Color::new(255, 0, 128),   // Hot Pink
                Color::new(255, 230, 0),   // Cyber Yellow
            ],
        ),
        "rainbow" => {
            let mut colors = Vec::with_capacity(8);
            for i in 0..8 {
                let hue = (i as f32 / 8.0) * 360.0;
                colors.push(Hsv::new(hue, 1.0, 1.0).to_rgb());
            }
            Palette::new("Rainbow", colors)
        }
        "sunset" => Palette::new(
            "Sunset",
            vec![
                Color::new(43, 0, 84),     // Midnight Purple
                Color::new(176, 38, 255),  // Electric Purple
                Color::new(255, 60, 56),   // Crimson Coral
                Color::new(255, 149, 0),   // Tangerine
                Color::new(255, 209, 102), // Sunset Gold
            ],
        ),
        "matrix" => Palette::new(
            "Matrix",
            vec![
                Color::new(0, 20, 0),      // Deep Forest
                Color::new(0, 140, 30),    // Code Green
                Color::new(50, 255, 100),  // Bright Matrix Green
                Color::new(200, 255, 220), // Glitch White-Green
            ],
        ),
        "ocean" => Palette::new(
            "Ocean",
            vec![
                Color::new(0, 30, 80),     // Deep Trench
                Color::new(0, 119, 182),   // Ocean Blue
                Color::new(72, 202, 228),  // Azure
                Color::new(144, 224, 239), // Sea Foam
            ],
        ),
        "fire" => Palette::new(
            "Fire",
            vec![
                Color::new(80, 0, 0),      // Dark Ember
                Color::new(200, 20, 0),    // Blaze Red
                Color::new(255, 100, 0),   // Fiery Orange
                Color::new(255, 220, 50),  // Bright Flame
            ],
        ),
        _ => get_palette("cyberpunk"),
    }
}
