use std::time::Duration;
use crate::color::Color;
use super::Animation;

#[derive(Debug, Clone)]
pub struct StaticColorAnimation {
    colors: [Color; 4],
    rendered: bool,
}

impl StaticColorAnimation {
    pub fn new(colors: [Color; 4]) -> Self {
        Self {
            colors,
            rendered: false,
        }
    }

    pub fn solid(color: Color) -> Self {
        Self::new([color; 4])
    }

    pub fn set_colors(&mut self, colors: [Color; 4]) {
        self.colors = colors;
        self.rendered = false;
    }

    pub fn set_zone(&mut self, zone: usize, color: Color) {
        if zone < 4 {
            self.colors[zone] = color;
            self.rendered = false;
        }
    }
}

impl Animation for StaticColorAnimation {
    fn name(&self) -> &'static str {
        "static"
    }

    /// Once rendered at least once, report false so the event loop timer stops immediately!
    fn is_animating(&self) -> bool {
        !self.rendered
    }

    fn tick(&mut self, _dt: Duration) -> [Color; 4] {
        self.rendered = true;
        self.colors
    }
}
