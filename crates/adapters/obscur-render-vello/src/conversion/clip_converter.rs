#![doc = include_str!("../../docs/api/conversion/clip_converter.doc/_module.md")]

use obscur_scene::clip::{Clip, ClipShape};
use obscur_scene::path::Path;
use vello::kurbo::{BezPath, Rect as KurboRect, Shape};

use super::path_converter::PathConverter;

/// Half-extent of the outline standing in for "no geometric bound". Vello has no
/// unbounded clip shape, so a clip that confines nothing is expressed as a rect
/// far larger than any real surface.
const UNBOUNDED_HALF_EXTENT: f64 = 1.0e9;

#[doc = include_str!("../../docs/api/conversion/clip_converter.doc/ClipConverter.md")]
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
            Clip::Mask(_) | Clip::ImageMask { .. } => Self::unbounded(),
        }
    }

    #[doc = include_str!("../../docs/api/conversion/clip_converter.doc/unbounded.md")]
    #[must_use]
    pub fn unbounded() -> BezPath {
        Self::unbounded_ref().clone()
    }

    #[doc = include_str!("../../docs/api/conversion/clip_converter.doc/unbounded_ref.md")]
    #[must_use]
    pub fn unbounded_ref() -> &'static BezPath {
        // Not a cache. The value depends on nothing — no argument, no state, no
        // frame — so there is no key it could be wrong about and nothing that could
        // invalidate it. It is a constant that `const` cannot express because
        // building it allocates. Callers that only need to *look* at the outline
        // (a layer with no clip: the common case, once per group-opacity layer per
        // frame) take this; `unbounded` above clones for callers that need an owned
        // path.
        static UNBOUNDED: std::sync::LazyLock<BezPath> = std::sync::LazyLock::new(|| {
            KurboRect::new(
                -UNBOUNDED_HALF_EXTENT,
                -UNBOUNDED_HALF_EXTENT,
                UNBOUNDED_HALF_EXTENT,
                UNBOUNDED_HALF_EXTENT,
            )
            .to_path(0.1)
        });
        &UNBOUNDED
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

    /// The borrowed outline and the owned one are the same shape — the whole
    /// point of building it once. A divergence here would clip differently
    /// depending on which accessor a call site happened to use.
    #[test]
    fn the_borrowed_unbounded_outline_is_the_owned_one() {
        assert_eq!(
            ClipConverter::unbounded_ref().elements(),
            ClipConverter::unbounded().elements()
        );
        // And it is genuinely one allocation, not one per call.
        assert!(std::ptr::eq(
            ClipConverter::unbounded_ref(),
            ClipConverter::unbounded_ref()
        ));
    }
}
