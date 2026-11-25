#![doc = include_str!("../../docs/api/paint/buffer_fit.doc/_module.md")]

use obscur_geometry::{Point, Rect, Size};

/// How a buffer is fitted to a region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BufferFit {
    /// Fill the region exactly, aspect ratio be damned. The default, and the right
    /// answer when the producer already renders at the region's size.
    #[default]
    Stretch,
    /// Scale to fit entirely inside, centred, letterboxed. Nothing is cropped.
    Contain,
    /// Scale to cover the whole region, centred; whatever overflows is clipped by the
    /// region itself. Nothing is letterboxed.
    Cover,
    /// Draw at the buffer's own pixel size, centred. No resampling — what a
    /// screenshot or a pixel-exact capture wants.
    Natural,
}

impl BufferFit {
    #[doc = include_str!("../../docs/api/paint/buffer_fit.doc/place.md")]
    #[must_use]
    pub fn place(self, source: Size, into: Size) -> Rect {
        let origin = Point::new(0.0, 0.0);
        if into.width <= 0.0 || into.height <= 0.0 {
            return Rect::new(origin, Size::new(0.0, 0.0));
        }
        // A source with no extent has no aspect ratio to preserve, so every fit
        // but `Stretch` would divide by nothing. Filling the region is the one
        // answer that is never wrong-looking.
        if source.width <= 0.0 || source.height <= 0.0 {
            return Rect::new(origin, into);
        }
        let scale = match self {
            Self::Stretch => return Rect::new(origin, into),
            Self::Contain => (into.width / source.width).min(into.height / source.height),
            Self::Cover => (into.width / source.width).max(into.height / source.height),
            Self::Natural => 1.0,
        };
        let size = Size::new(source.width * scale, source.height * scale);
        // Centred, and centred the same way when it overflows: a `Cover` frame
        // hangs off both edges equally rather than being pinned to a corner.
        Rect::new(
            Point::new(
                (into.width - size.width) / 2.0,
                (into.height - size.height) / 2.0,
            ),
            size,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::BufferFit;
    use obscur_geometry::Size;

    fn source() -> Size {
        Size::new(200.0, 100.0)
    }

    fn region() -> Size {
        Size::new(100.0, 100.0)
    }

    #[test]
    fn stretch_fills_the_region_and_forgets_the_aspect_ratio() {
        let placed = BufferFit::Stretch.place(source(), region());
        assert_eq!(placed.origin.x, 0.0);
        assert_eq!(placed.size, region());
    }

    #[test]
    fn contain_fits_inside_and_centres_what_is_left_over() {
        let placed = BufferFit::Contain.place(source(), region());
        assert_eq!(placed.size, Size::new(100.0, 50.0));
        assert_eq!(placed.origin.y, 25.0, "the letterbox is split evenly");
        assert_eq!(placed.origin.x, 0.0);
    }

    #[test]
    fn cover_fills_the_region_and_hangs_off_both_edges_equally() {
        // Both edges, not a corner: a frame pinned to the top-left crops one
        // side twice and the other not at all.
        let placed = BufferFit::Cover.place(source(), region());
        assert_eq!(placed.size, Size::new(200.0, 100.0));
        assert_eq!(placed.origin.x, -50.0);
        assert_eq!(placed.origin.y, 0.0);
    }

    #[test]
    fn natural_draws_the_buffer_at_its_own_size() {
        // What a screenshot or a pixel-exact capture wants: no resampling.
        let placed = BufferFit::Natural.place(Size::new(40.0, 20.0), region());
        assert_eq!(placed.size, Size::new(40.0, 20.0));
        assert_eq!(placed.origin.x, 30.0);
        assert_eq!(placed.origin.y, 40.0);
    }

    #[test]
    fn a_region_with_no_extent_places_nothing() {
        for fit in [BufferFit::Stretch, BufferFit::Contain, BufferFit::Cover] {
            assert_eq!(fit.place(source(), Size::new(0.0, 50.0)).size.width, 0.0);
        }
    }

    #[test]
    fn a_source_with_no_extent_fills_rather_than_dividing_by_nothing() {
        // A camera that has not produced a frame yet is a real state.
        assert_eq!(
            BufferFit::Contain.place(Size::new(0.0, 0.0), region()).size,
            region()
        );
    }
}
