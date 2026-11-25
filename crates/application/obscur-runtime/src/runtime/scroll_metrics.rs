//! Live scroll geometry exposed by the runtime.

use obscur_geometry::{Point, Size};

/// A snapshot of an arranged scroll container.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollMetrics {
    /// Current clamped scroll offset in logical pixels.
    pub offset: Point,
    /// Visible size of the scrollport in logical pixels.
    pub viewport: Size,
    /// Total extent of scrollable content in logical pixels.
    pub content_extent: Size,
}
