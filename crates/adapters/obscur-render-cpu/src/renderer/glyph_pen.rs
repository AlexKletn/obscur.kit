//! A [`skrifa`] [`OutlinePen`] that accumulates a glyph's scaled outline into a
//! [`tiny_skia::Path`], placed at the glyph's device-space pen position (spec
//! §5.2 real glyph rasterization).
//!
//! `skrifa` emits outline coordinates already scaled to pixels (via
//! [`Size`](skrifa::instance::Size)) in the font's y-up, baseline-at-origin
//! coordinate system. This pen flips the y axis and translates each point onto
//! the glyph's baseline pen position `(pen_x, baseline_y)` in the pixmap's
//! y-down device space, so the resulting path is ready to fill directly.

use skrifa::outline::OutlinePen;

/// Collects a single glyph's outline into a device-space [`tiny_skia::Path`].
///
/// Constructed with the glyph's baseline pen position; each pen callback maps a
/// scaled font-space point `(x, y)` (y up from the baseline) to the device point
/// `(pen_x + x + skew * y, baseline_y - y)` (y down). The `skew` factor is the
/// synthetic-oblique shear (spec §5.18): with `skew == 0` the map is the plain
/// upright placement `(pen_x + x, baseline_y - y)`; a positive `skew` slides
/// above-baseline points rightward so the glyph leans forward, faking an italic.
/// [`GlyphPathPen::into_path`] finishes the builder, yielding `None` for a glyph
/// with no contours (whitespace, glyph 0, or a degenerate outline `tiny-skia`
/// rejects) so it draws nothing.
pub(crate) struct GlyphPathPen {
    /// The path under construction, in device space.
    builder: tiny_skia::PathBuilder,
    /// The glyph's baseline pen x in device space.
    pen_x: f32,
    /// The glyph's baseline y in device space.
    baseline_y: f32,
    /// The synthetic-oblique shear factor (`0.0` = upright); applied per point as
    /// a rightward x-shift proportional to the point's height above the baseline.
    skew: f32,
    /// Whether any contour command was emitted (empty glyphs draw nothing).
    has_contour: bool,
}

impl GlyphPathPen {
    /// Creates an upright pen that places the glyph's outline at baseline pen
    /// position `(pen_x, baseline_y)` in the pixmap's y-down device space.
    pub(crate) fn new(pen_x: f32, baseline_y: f32) -> Self {
        Self::with_skew(pen_x, baseline_y, 0.0)
    }

    /// Creates a pen that additionally shears the outline by `skew` to synthesize
    /// an oblique slant (spec §5.18): each point's device x is shifted right by
    /// `skew` times its scaled height above the baseline, so the glyph leans
    /// forward. A `skew` of `0.0` is byte-identical to [`GlyphPathPen::new`].
    pub(crate) fn with_skew(pen_x: f32, baseline_y: f32, skew: f32) -> Self {
        Self {
            builder: tiny_skia::PathBuilder::new(),
            pen_x,
            baseline_y,
            skew,
            has_contour: false,
        }
    }

    /// Finishes the accumulated outline into a [`tiny_skia::Path`], or `None`
    /// when the glyph emitted no contours or the outline is degenerate.
    pub(crate) fn into_path(self) -> Option<tiny_skia::Path> {
        if !self.has_contour {
            return None;
        }
        self.builder.finish()
    }

    /// Maps a scaled font-space point `(x, y)` (y up) onto device x, shearing by
    /// `skew * y` so above-baseline points slide right (the oblique lean).
    fn dx(&self, x: f32, y: f32) -> f32 {
        self.pen_x + x + self.skew * y
    }

    /// Maps a scaled font-space y (up) onto device space (down).
    fn dy(&self, y: f32) -> f32 {
        self.baseline_y - y
    }
}

impl OutlinePen for GlyphPathPen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.has_contour = true;
        self.builder.move_to(self.dx(x, y), self.dy(y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.builder.line_to(self.dx(x, y), self.dy(y));
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        self.builder
            .quad_to(self.dx(cx0, cy0), self.dy(cy0), self.dx(x, y), self.dy(y));
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.builder.cubic_to(
            self.dx(cx0, cy0),
            self.dy(cy0),
            self.dx(cx1, cy1),
            self.dy(cy1),
            self.dx(x, y),
            self.dy(y),
        );
    }

    fn close(&mut self) {
        self.builder.close();
    }
}
