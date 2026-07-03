//! The geometry a clip restricts drawing to.

use obscur_geometry::{CornerRadii, Rect};

use crate::path::Path;

/// The geometric region a clip confines subsequent drawing to (spec §5.8).
///
/// A `ClipShape` is pure, backend-independent geometry: a plain rectangle, a
/// rounded rectangle, or an arbitrary [`Path`]. It carries no rendering state
/// and references no GPU or renderer types. Rectangular and rounded-rectangular
/// shapes are cheap to apply; a path shape is more general but still describes
/// only geometry.
#[derive(Debug, Clone, PartialEq)]
pub enum ClipShape {
    /// A plain axis-aligned rectangle.
    Rect(Rect),
    /// A rectangle with per-corner rounding.
    RoundedRect {
        /// The rectangle being rounded.
        rect: Rect,
        /// The per-corner radii applied to `rect`.
        radii: CornerRadii,
    },
    /// An arbitrary vector outline.
    Path(Path),
}

impl ClipShape {
    /// Creates a rectangular clip shape.
    #[must_use]
    pub const fn rect(rect: Rect) -> Self {
        Self::Rect(rect)
    }

    /// Creates a rounded-rectangular clip shape from a rectangle and its
    /// per-corner radii.
    #[must_use]
    pub const fn rounded_rect(rect: Rect, radii: CornerRadii) -> Self {
        Self::RoundedRect { rect, radii }
    }

    /// Creates a clip shape from an arbitrary path outline.
    #[must_use]
    pub fn path(path: Path) -> Self {
        Self::Path(path)
    }

    /// The shape's axis-aligned bounding rectangle.
    ///
    /// For [`ClipShape::Rect`] and [`ClipShape::RoundedRect`] this is the
    /// rectangle itself (rounding only removes area, so the outer rectangle
    /// still bounds the shape). For [`ClipShape::Path`] this delegates to
    /// [`Path::bounds`], which returns `None` for an empty path. Never panics.
    #[must_use]
    pub fn bounds(&self) -> Option<Rect> {
        match self {
            Self::Rect(rect) => Some(*rect),
            Self::RoundedRect { rect, .. } => Some(*rect),
            Self::Path(path) => path.bounds(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::{Point, Size};

    fn sample_rect() -> Rect {
        Rect::new(Point::new(1.0, 2.0), Size::new(10.0, 20.0))
    }

    #[test]
    fn rect_bounds_is_the_rect() {
        let shape = ClipShape::rect(sample_rect());
        assert_eq!(shape.bounds(), Some(sample_rect()));
    }

    #[test]
    fn rounded_rect_bounds_is_the_outer_rect() {
        let shape = ClipShape::rounded_rect(sample_rect(), CornerRadii::uniform(4.0));
        assert_eq!(shape.bounds(), Some(sample_rect()));
    }

    #[test]
    fn path_bounds_delegates_to_path() {
        let mut path = Path::new();
        path.move_to(Point::new(0.0, 0.0))
            .line_to(Point::new(5.0, 8.0));
        let shape = ClipShape::path(path.clone());
        assert_eq!(shape.bounds(), path.bounds());
    }

    #[test]
    fn empty_path_bounds_is_none() {
        let shape = ClipShape::path(Path::new());
        assert_eq!(shape.bounds(), None);
    }

    #[test]
    fn constructors_match_variants() {
        assert!(matches!(ClipShape::rect(sample_rect()), ClipShape::Rect(_)));
        assert!(matches!(
            ClipShape::rounded_rect(sample_rect(), CornerRadii::uniform(1.0)),
            ClipShape::RoundedRect { .. }
        ));
    }
}
