//! `TaffyResultMapper`: translates a computed `taffy::Layout` back into an obscur
//! [`LayoutBox`].
//!
//! Part of the internal, spec-sanctioned Taffy provider (spec §3.6: Taffy stays an
//! internal implementation detail). See [`super::TaffyStyleMapper`] for the inbound
//! direction and the §3.6 boundary discussion.

// Stateless value object whose bridge method is spec-named to take `&self`
// (spec §3.6). The pedantic self-convention/`unused_self` lints do not apply.
#![allow(
    clippy::unused_self,
    clippy::trivially_copy_pass_by_ref,
    clippy::wrong_self_convention
)]

use crate::model::LayoutBox;
use obscur_geometry::{Point, Rect, Size};

/// Maps a computed `taffy::Layout` to an obscur [`LayoutBox`].
///
/// Stateless: construct with [`TaffyResultMapper::new`] (or `Default`) and reuse.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TaffyResultMapper;

impl TaffyResultMapper {
    /// Creates a new, stateless result mapper.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Converts a computed `taffy::Layout` into an obscur [`LayoutBox`].
    ///
    /// The taffy location becomes the box origin and the taffy size becomes the box
    /// size; no rounding is applied (fractional geometry is preserved, spec §4.6).
    ///
    /// ## §3.6
    ///
    /// This method is `pub(crate)`: it takes a `taffy::Layout`, so it must stay out of
    /// the crate's public API. Only [`super::TaffyTreeView`] invokes it, and that type
    /// exposes exclusively obscur types.
    #[must_use]
    pub(crate) fn to_layout_box(&self, layout: &taffy::Layout) -> LayoutBox {
        LayoutBox::new(Rect::new(
            Point::new(layout.location.x, layout.location.y),
            Size::new(layout.size.width, layout.size.height),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::TaffyResultMapper;
    use obscur_geometry::{Point, Size};

    #[test]
    fn maps_location_and_size() {
        let mut layout = taffy::Layout::with_order(0);
        layout.location = taffy::Point { x: 10.0, y: 20.0 };
        layout.size = taffy::Size {
            width: 30.0,
            height: 40.0,
        };

        let box_ = TaffyResultMapper::new().to_layout_box(&layout);
        assert_eq!(box_.origin(), Point::new(10.0, 20.0));
        assert_eq!(box_.size(), Size::new(30.0, 40.0));
    }
}
