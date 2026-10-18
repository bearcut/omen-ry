pub mod mock;
pub mod sysfs;

use std::io;
use crate::color::Color;

pub trait ZoneBackend: Send {
    /// Update the colors across the 4 keyboard zones
    fn set_zones(&mut self, colors: &[Color; 4]) -> io::Result<()>;

    /// Set global brightness (0 - 100%)
    fn set_brightness(&mut self, brightness: u8) -> io::Result<()>;

    /// Human-readable backend name
    fn backend_name(&self) -> &'static str;
}
