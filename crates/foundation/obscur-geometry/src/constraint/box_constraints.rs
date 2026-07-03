//! Min/max box constraints for the measure pass.

use crate::primitive::Size;

/// Lower and upper bounds on a box's logical size, passed down during the
/// measure phase of layout (spec §4.2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoxConstraints {
    /// Minimum size.
    pub min: Size,
    /// Maximum size. Components may be [`f32::INFINITY`] for unbounded axes.
    pub max: Size,
}

impl BoxConstraints {
    /// Constraints that admit any size from zero up to `max`.
    #[must_use]
    pub const fn loose(max: Size) -> Self {
        Self {
            min: Size::ZERO,
            max,
        }
    }

    /// Constraints that force exactly `size`.
    #[must_use]
    pub const fn tight(size: Size) -> Self {
        Self {
            min: size,
            max: size,
        }
    }

    /// Clamps a desired size into these constraints.
    #[must_use]
    pub fn constrain(self, desired: Size) -> Size {
        Size::new(
            desired.width.clamp(self.min.width, self.max.width),
            desired.height.clamp(self.min.height, self.max.height),
        )
    }
}
