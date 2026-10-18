pub mod static_color;
pub mod wave;

use std::time::Duration;
use crate::color::Color;

pub trait Animation: Send + Sync {
    /// Identifier name for this animation mode
    fn name(&self) -> &'static str;

    /// Whether this animation actively requires periodic timer ticks.
    /// When this returns false, the event loop stops the timer, dropping to 0.0% CPU!
    fn is_animating(&self) -> bool;

    /// Calculate the color for all 4 zones for the current frame
    fn tick(&mut self, dt: Duration) -> [Color; 4];
}
