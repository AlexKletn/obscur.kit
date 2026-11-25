#![doc = include_str!("../../docs/api/renderer/glyph_pen.doc/_module.md")]

use skrifa::outline::OutlinePen;

#[doc = include_str!("../../docs/api/renderer/glyph_pen.doc/GlyphPathPen.md")]
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

/// Appends an expanded glyph outline to the run path for one synthetic-bold fill.
#[allow(clippy::cast_possible_truncation)]
pub(super) fn append_bold_outline(
    destination: &mut tiny_skia::PathBuilder,
    outline: &tiny_skia::Path,
    font_size: f32,
) {
    use kurbo::PathEl;
    use tiny_skia::PathSegment;

    let point = |p: tiny_skia::Point| kurbo::Point::new(f64::from(p.x), f64::from(p.y));
    let outline: kurbo::BezPath = outline
        .segments()
        .map(|segment| match segment {
            PathSegment::MoveTo(p) => PathEl::MoveTo(point(p)),
            PathSegment::LineTo(p) => PathEl::LineTo(point(p)),
            PathSegment::QuadTo(a, p) => PathEl::QuadTo(point(a), point(p)),
            PathSegment::CubicTo(a, b, p) => PathEl::CurveTo(point(a), point(b), point(p)),
            PathSegment::Close => PathEl::ClosePath,
        })
        .collect();
    let amount =
        f64::from(font_size) * f64::from(obscur_scene::text::SYNTHETIC_BOLD_OUTSET_FRACTION);
    let expanded = kurbo::expand_path(
        &outline,
        kurbo::Diagonal2::new(amount, amount),
        kurbo::Join::Miter,
        4.0,
        0.1,
    );
    for &element in expanded.elements() {
        match element {
            PathEl::MoveTo(p) => destination.move_to(p.x as f32, p.y as f32),
            PathEl::LineTo(p) => destination.line_to(p.x as f32, p.y as f32),
            PathEl::QuadTo(a, p) => {
                destination.quad_to(a.x as f32, a.y as f32, p.x as f32, p.y as f32)
            }
            PathEl::CurveTo(a, b, p) => destination.cubic_to(
                a.x as f32, a.y as f32, b.x as f32, b.y as f32, p.x as f32, p.y as f32,
            ),
            PathEl::ClosePath => destination.close(),
        }
    }
}
