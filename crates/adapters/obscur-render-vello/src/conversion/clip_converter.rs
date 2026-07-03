//! Converts an obscur-scene [`Clip`] into a kurbo [`BezPath`] clip shape.
//!
//! Pure, backend-local conversion (spec §2.7). Vello models a clip as a layer
//! bounded by a shape, so a clip becomes the [`BezPath`] outline it confines
//! drawing to. Geometry is preserved exactly (spec §5.2).

use obscur_scene::clip::{Clip, ClipShape};
use obscur_scene::path::Path;
use vello::kurbo::BezPath;

use super::path_converter::PathConverter;

/// A converter from an obscur-scene [`Clip`] to a kurbo [`BezPath`] clip shape.
///
/// It owns a [`PathConverter`] so every clip outline is built through the same
/// path conversion.
///
/// # Shape mapping
///
/// - [`ClipShape::Rect`] → the rectangle's four-line closed outline
///   (via [`Path::rect`]).
/// - [`ClipShape::RoundedRect`] → the rounded-rectangle outline
///   (via [`Path::rounded_rect`]), matching the domain's own rounding.
/// - [`ClipShape::Path`] → the path converted directly.
/// - [`Clip::Mask`] → an **empty** `BezPath`: a mask clip derives coverage from
///   separate source content and has no geometric outline, so it cannot be
///   expressed as a clip shape here. Mask coverage is handled outside this pure
///   geometric conversion; an empty shape clips away nothing.
#[derive(Debug, Default, Clone, Copy)]
pub struct ClipConverter {
    path: PathConverter,
}

impl ClipConverter {
    /// Creates a clip converter using the given path converter.
    #[must_use]
    pub const fn new(path: PathConverter) -> Self {
        Self { path }
    }

    /// Converts an obscur-scene [`Clip`] into a kurbo [`BezPath`] clip shape.
    #[must_use]
    pub fn shape(&self, clip: &Clip) -> BezPath {
        match clip {
            Clip::Shape(ClipShape::Rect(rect)) => self.path.convert(&Path::rect(*rect)),
            Clip::Shape(ClipShape::RoundedRect { rect, radii }) => {
                self.path.convert(&Path::rounded_rect(*rect, *radii))
            }
            Clip::Shape(ClipShape::Path(path)) => self.path.convert(path),
            Clip::Mask(_) => BezPath::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::{CornerRadii, Point, Rect, Size};
    use obscur_scene::clip::Mask;
    use vello::kurbo::PathEl;

    #[test]
    fn rect_clip_becomes_closed_rectangle() {
        let clip = Clip::rect(Rect::new(Point::new(1.0, 2.0), Size::new(4.0, 6.0)));
        let bez = ClipConverter::default().shape(&clip);
        assert_eq!(bez.elements().len(), 5);
        assert!(matches!(bez.elements()[0], PathEl::MoveTo(_)));
        assert!(matches!(bez.elements()[4], PathEl::ClosePath));
    }

    #[test]
    fn rounded_rect_clip_is_non_empty() {
        let clip = Clip::rounded_rect(
            Rect::new(Point::new(0.0, 0.0), Size::new(20.0, 10.0)),
            CornerRadii::uniform(3.0),
        );
        let bez = ClipConverter::default().shape(&clip);
        assert!(!bez.elements().is_empty());
    }

    #[test]
    fn mask_clip_is_empty_shape() {
        let bez = ClipConverter::default().shape(&Clip::mask(Mask::luminance()));
        assert!(bez.elements().is_empty());
    }
}
