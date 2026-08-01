#![doc = include_str!("logical_px.doc/_module.md")]

use super::{PhysicalPx, ScaleFactor};

#[doc = include_str!("logical_px.doc/LogicalPx.md")]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct LogicalPx(pub f32);

impl LogicalPx {
    #[doc = include_str!("logical_px.doc/to_physical.md")]
    #[must_use]
    pub fn to_physical(self, scale: ScaleFactor) -> PhysicalPx {
        PhysicalPx(self.0 * scale.get())
    }
}
