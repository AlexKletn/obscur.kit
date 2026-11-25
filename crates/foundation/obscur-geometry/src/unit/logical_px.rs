//! Logical (device-independent) pixel scalar.

use super::{PhysicalPx, ScaleFactor};

/// A length in logical pixels. Layout works exclusively in logical units;
/// conversion to physical pixels happens at render time (spec §4.5).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct LogicalPx(pub f32);

impl LogicalPx {
    /// Converts to physical pixels using the given scale factor.
    #[must_use]
    pub fn to_physical(self, scale: ScaleFactor) -> PhysicalPx {
        PhysicalPx(self.0 * scale.get())
    }
}
