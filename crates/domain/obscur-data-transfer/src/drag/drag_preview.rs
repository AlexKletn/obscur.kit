//! The visual drag-image placement descriptor (spec §6.21 drag and drop —
//! previews).

use obscur_geometry::{Point, Size};

#[doc = include_str!("../../docs/api/drag/drag_preview.doc/DragPreview.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragPreview {
    size: Size,
    cursor_offset: Point,
    opacity: f32,
}

impl DragPreview {
    /// Creates a preview of `size` with the hotspot at its origin and full
    /// opacity (spec §6.21).
    #[must_use]
    pub const fn new(size: Size) -> Self {
        Self {
            size,
            cursor_offset: Point::new(0.0, 0.0),
            opacity: 1.0,
        }
    }

    /// Returns this preview with the pointer hotspot set to `cursor_offset`
    /// (spec §6.21).
    #[must_use]
    pub const fn with_cursor_offset(mut self, cursor_offset: Point) -> Self {
        self.cursor_offset = cursor_offset;
        self
    }

    /// Returns this preview with `opacity` clamped to `[0.0, 1.0]` (spec §6.21).
    #[must_use]
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// Returns the preview's logical-pixel size (spec §6.21).
    #[must_use]
    pub const fn size(&self) -> Size {
        self.size
    }

    /// Returns the pointer hotspot within the preview (spec §6.21).
    #[must_use]
    pub const fn cursor_offset(&self) -> Point {
        self.cursor_offset
    }

    /// Returns the preview's opacity, in `[0.0, 1.0]` (spec §6.21).
    #[must_use]
    pub const fn opacity(&self) -> f32 {
        self.opacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_has_origin_hotspot_and_full_opacity() {
        let preview = DragPreview::new(Size::new(64.0, 48.0));
        assert_eq!(preview.size(), Size::new(64.0, 48.0));
        assert_eq!(preview.cursor_offset(), Point::new(0.0, 0.0));
        assert!((preview.opacity() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn with_cursor_offset_sets_hotspot() {
        let preview =
            DragPreview::new(Size::new(64.0, 48.0)).with_cursor_offset(Point::new(8.0, 4.0));
        assert_eq!(preview.cursor_offset(), Point::new(8.0, 4.0));
    }

    #[test]
    fn with_opacity_clamps_to_unit_range() {
        let preview = DragPreview::new(Size::new(10.0, 10.0));
        assert!((preview.with_opacity(0.5).opacity() - 0.5).abs() < 1e-6);
        assert!((preview.with_opacity(2.0).opacity() - 1.0).abs() < 1e-6);
        assert!(preview.with_opacity(-1.0).opacity().abs() < 1e-6);
    }
}
