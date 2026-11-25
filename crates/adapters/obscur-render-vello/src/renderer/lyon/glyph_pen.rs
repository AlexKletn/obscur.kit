use obscur_geometry::Point;
use obscur_scene::path::Path;
use skrifa::outline::OutlinePen;

pub(super) struct GlyphPen {
    pub(super) path: Path,
    pub(super) shear: f32,
    pub(super) y_scale: f32,
}

impl GlyphPen {
    fn point(&self, x: f32, y: f32) -> Point {
        Point::new(self.shear.mul_add(y, x), self.y_scale * y)
    }
}

impl OutlinePen for GlyphPen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.path.move_to(self.point(x, y));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.path.line_to(self.point(x, y));
    }
    fn quad_to(&mut self, x: f32, y: f32, x1: f32, y1: f32) {
        self.path.quad_to(self.point(x, y), self.point(x1, y1));
    }
    fn curve_to(&mut self, x: f32, y: f32, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.path
            .cubic_to(self.point(x, y), self.point(x1, y1), self.point(x2, y2));
    }
    fn close(&mut self) {
        self.path.close();
    }
}
