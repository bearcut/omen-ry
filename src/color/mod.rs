pub mod palettes;

use std::fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255 };
    pub const RED: Color = Color { r: 255, g: 0, b: 0 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255 };

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Parse a 6-character hex string (e.g. "FF00AA" or "#FF00AA")
    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let clean = hex.trim().trim_start_matches('#');
        if clean.len() != 6 {
            return Err(format!("Invalid hex color length (expected 6, got {}): '{}'", clean.len(), hex));
        }

        let r = u8::from_str_radix(&clean[0..2], 16)
            .map_err(|e| format!("Invalid red hex component: {}", e))?;
        let g = u8::from_str_radix(&clean[2..4], 16)
            .map_err(|e| format!("Invalid green hex component: {}", e))?;
        let b = u8::from_str_radix(&clean[4..6], 16)
            .map_err(|e| format!("Invalid blue hex component: {}", e))?;

        Ok(Self { r, g, b })
    }

    /// Formats as 6 uppercase hex digits without '#' (format expected by sysfs)
    pub fn to_hex(&self) -> String {
        format!("{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// Formats with a leading '#'
    pub fn to_hex_hash(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// Linear interpolation between two colors
    pub fn lerp(&self, other: &Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        let r = (self.r as f32 + (other.r as f32 - self.r as f32) * t).round() as u8;
        let g = (self.g as f32 + (other.g as f32 - self.g as f32) * t).round() as u8;
        let b = (self.b as f32 + (other.b as f32 - self.b as f32) * t).round() as u8;
        Color { r, g, b }
    }

    /// Applies brightness scaling (0 - 100%)
    pub fn scale_brightness(&self, brightness: u8) -> Color {
        let scale = brightness.min(100) as f32 / 100.0;
        Color {
            r: (self.r as f32 * scale).round() as u8,
            g: (self.g as f32 * scale).round() as u8,
            b: (self.b as f32 * scale).round() as u8,
        }
    }

    /// Returns ANSI 24-bit truecolor escape sequence for background block
    pub fn to_ansi_bg(&self) -> String {
        format!("\x1b[48;2;{};{};{}m  \x1b[0m", self.r, self.g, self.b)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex_hash())
    }
}

/// HSV representation for smooth rainbow cycles
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsv {
    pub h: f32, // 0.0 .. 360.0
    pub s: f32, // 0.0 .. 1.0
    pub v: f32, // 0.0 .. 1.0
}

impl Hsv {
    pub fn new(h: f32, s: f32, v: f32) -> Self {
        Self {
            h: h.rem_euclid(360.0),
            s: s.clamp(0.0, 1.0),
            v: v.clamp(0.0, 1.0),
        }
    }

    pub fn to_rgb(&self) -> Color {
        let c = self.v * self.s;
        let x = c * (1.0 - ((self.h / 60.0).rem_euclid(2.0) - 1.0).abs());
        let m = self.v - c;

        let (r1, g1, b1) = match self.h as u32 {
            0..=59 => (c, x, 0.0),
            60..=119 => (x, c, 0.0),
            120..=179 => (0.0, c, x),
            180..=239 => (0.0, x, c),
            240..=299 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Color {
            r: ((r1 + m) * 255.0).round() as u8,
            g: ((g1 + m) * 255.0).round() as u8,
            b: ((b1 + m) * 255.0).round() as u8,
        }
    }
}
