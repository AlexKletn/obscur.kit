#![doc = include_str!("glyph_pen.doc/_module.md")]

use skrifa::outline::OutlinePen;

#[doc = include_str!("glyph_pen.doc/GlyphPathPen.md")]
pub(crate) struct GlyphPathPen {
    #[doc = include_str!("glyph_pen.doc/builder.md")]
    builder: tiny_skia::PathBuilder,
    #[doc = include_str!("glyph_pen.doc/pen_x.md")]
    pen_x: f32,
    #[doc = include_str!("glyph_pen.doc/baseline_y.md")]
    baseline_y: f32,
    #[doc = include_str!("glyph_pen.doc/skew.md")]
    skew: f32,
    #[doc = include_str!("glyph_pen.doc/has_contour.md")]
    has_contour: bool,
}

impl GlyphPathPen {
    #[doc = include_str!("glyph_pen.doc/new.md")]
    pub(crate) fn new(pen_x: f32, baseline_y: f32) -> Self {
        Self::with_skew(pen_x, baseline_y, 0.0)
    }

    #[doc = include_str!("glyph_pen.doc/with_skew.md")]
    pub(crate) fn with_skew(pen_x: f32, baseline_y: f32, skew: f32) -> Self {
        Self {
            builder: tiny_skia::PathBuilder::new(),
            pen_x,
            baseline_y,
            skew,
            has_contour: false,
        }
    }

    #[doc = include_str!("glyph_pen.doc/into_path.md")]
    pub(crate) fn into_path(self) -> Option<tiny_skia::Path> {
        if !self.has_contour {
            return None;
        }
        self.builder.finish()
    }

    #[doc = include_str!("glyph_pen.doc/dx.md")]
    fn dx(&self, x: f32, y: f32) -> f32 {
        self.pen_x + x + self.skew * y
    }

    #[doc = include_str!("glyph_pen.doc/dy.md")]
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
