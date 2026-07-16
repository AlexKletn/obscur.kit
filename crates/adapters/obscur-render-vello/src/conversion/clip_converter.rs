//! Converts an obscur-scene [`Clip`] into a kurbo [`BezPath`] clip shape.
//!
//! Pure, backend-local conversion (spec §2.7). Vello models a clip as a layer
//! bounded by a shape, so a clip becomes the [`BezPath`] outline it confines
//! drawing to. Geometry is preserved exactly (spec §5.2).

use obscur_scene::clip::{Clip, ClipShape};
use obscur_scene::path::Path;
use vello::kurbo::{BezPath, Rect as KurboRect, Shape};

use super::path_converter::PathConverter;

/// Half-extent of the outline standing in for "no geometric bound". Vello has no
/// unbounded clip shape, so a clip that confines nothing is expressed as a rect
/// far larger than any real surface.
const UNBOUNDED_HALF_EXTENT: f64 = 1.0e9;

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
/// - [`Clip::Mask`] → the [`unbounded`](Self::unbounded) outline, which confines
///   nothing. A mask carries only *how* to read its coverage, never the source
///   the coverage comes from ([`Mask`](obscur_scene::clip::Mask)), so a mask clip
///   in the command stream has no known extent — [`Clip::bounds`] reports `None`
///   for exactly this reason. An empty `BezPath` would be the opposite claim
///   (extent known, and empty), and Vello would clip the content away entirely;
///   inheriting the enclosing clip asserts nothing the domain has not stated, and
///   matches the CPU backend (spec §5.2 requires clipping to match across
///   backends).
///
///   This is the documented §5.8 fallback, **not** mask support: coverage is not
///   applied, so a masked region paints as if unmasked on both backends. Neither
///   backend rasterizes mask coverage today, and neither raises the §5.8
///   diagnostic that an explicit fallback is supposed to carry.
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
            Clip::Mask(_) => Self::unbounded(),
        }
    }

    /// The outline standing in for "no geometric bound": a rect far larger than
    /// any real surface, so drawing through it is confined by nothing.
    ///
    /// Used for a [`Clip::Mask`] and for a layer with no clip at all — both cases
    /// where nothing is known to confine the content.
    #[must_use]
    pub fn unbounded() -> BezPath {
        KurboRect::new(
            -UNBOUNDED_HALF_EXTENT,
            -UNBOUNDED_HALF_EXTENT,
            UNBOUNDED_HALF_EXTENT,
            UNBOUNDED_HALF_EXTENT,
        )
        .to_path(0.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::{CornerRadii, Point, Rect, Size};
    use obscur_scene::clip::Mask;
    use vello::kurbo::{PathEl, Point as KurboPoint};

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

    /// A mask carries no source and no extent, so it must confine nothing. An
    /// empty shape would clip the content away — the opposite of the CPU backend
    /// (spec §5.2).
    #[test]
    fn mask_clip_confines_nothing() {
        for mask in [Mask::luminance(), Mask::alpha()] {
            let bez = ClipConverter::default().shape(&Clip::mask(mask));
            assert!(!bez.elements().is_empty(), "{mask:?} produced no outline");
            assert!(
                bez.contains(KurboPoint::new(0.0, 0.0)),
                "{mask:?} clipped away the origin"
            );
            assert!(
                bez.contains(KurboPoint::new(4096.0, 4096.0)),
                "{mask:?} clipped away a far corner"
            );
        }
    }

    /// A layer with no clip and a mask clip are the same "nothing is known to
    /// confine this" case, and must resolve identically.
    #[test]
    fn mask_clip_matches_the_unbounded_outline() {
        let bez = ClipConverter::default().shape(&Clip::mask(Mask::alpha()));
        assert_eq!(bez.elements(), ClipConverter::unbounded().elements());
    }
}
