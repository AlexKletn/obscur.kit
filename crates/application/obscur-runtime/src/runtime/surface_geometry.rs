#![doc = include_str!("surface_geometry.doc/_module.md")]

use obscur_geometry::{AffineTransform, Point, Rect, Size};
use obscur_resources::source::requested_density;

#[doc = include_str!("surface_geometry.doc/MIN_DEVICE_SCALE.md")]
const MIN_DEVICE_SCALE: f32 = 0.1;

#[doc = include_str!("surface_geometry.doc/SurfaceGeometry.md")]
#[derive(Debug, Clone, Copy)]
pub(crate) struct SurfaceGeometry {
    #[doc = include_str!("surface_geometry.doc/viewport.md")]
    viewport: Size,
    #[doc = include_str!("surface_geometry.doc/viewport_scroll.md")]
    viewport_scroll: (f32, f32),
    #[doc = include_str!("surface_geometry.doc/device_scale.md")]
    device_scale: f32,
    #[doc = include_str!("surface_geometry.doc/content_offset.md")]
    content_offset: (f32, f32),
}

#[doc = include_str!("surface_geometry.doc/SurfaceGeometry_2.md")]
impl SurfaceGeometry {
    #[doc = include_str!("surface_geometry.doc/new.md")]
    pub(crate) const fn new(viewport: Size) -> Self {
        Self {
            viewport,
            viewport_scroll: (0.0, 0.0),
            device_scale: 1.0,
            content_offset: (0.0, 0.0),
        }
    }

    #[doc = include_str!("surface_geometry.doc/viewport_2.md")]
    pub(crate) const fn viewport(self) -> Size {
        self.viewport
    }

    #[doc = include_str!("surface_geometry.doc/set_viewport.md")]
    pub(crate) const fn set_viewport(&mut self, viewport: Size) {
        self.viewport = viewport;
    }

    #[doc = include_str!("surface_geometry.doc/device_scale_2.md")]
    pub(crate) const fn device_scale(self) -> f32 {
        self.device_scale
    }

    #[doc = include_str!("surface_geometry.doc/set_device_scale.md")]
    pub(crate) fn set_device_scale(&mut self, scale: f32) -> bool {
        let previous = self.device_scale;
        self.device_scale = if scale.is_finite() {
            scale.max(MIN_DEVICE_SCALE)
        } else {
            MIN_DEVICE_SCALE
        };
        requested_density(previous) != requested_density(self.device_scale)
    }

    #[doc = include_str!("surface_geometry.doc/set_surface.md")]
    pub(crate) fn set_surface(&mut self, physical: Size, scale: f32) -> bool {
        let rebanded = self.set_device_scale(scale);
        let effective = self.device_scale;
        self.viewport = Size::new(physical.width / effective, physical.height / effective);
        rebanded
    }

    #[doc = include_str!("surface_geometry.doc/content_offset_2.md")]
    pub(crate) const fn content_offset(self) -> (f32, f32) {
        self.content_offset
    }

    #[doc = include_str!("surface_geometry.doc/set_content_offset.md")]
    pub(crate) fn set_content_offset(&mut self, x: f32, y: f32) {
        let (ox, oy) = self.content_offset;
        self.content_offset = (
            if x.is_finite() { x } else { ox },
            if y.is_finite() { y } else { oy },
        );
    }

    #[doc = include_str!("surface_geometry.doc/viewport_scroll_2.md")]
    pub(crate) const fn viewport_scroll(self) -> (f32, f32) {
        self.viewport_scroll
    }

    #[doc = include_str!("surface_geometry.doc/set_viewport_scroll.md")]
    pub(crate) const fn set_viewport_scroll(&mut self, x: f32, y: f32) {
        self.viewport_scroll = (x, y);
    }

    #[doc = include_str!("surface_geometry.doc/document_viewport.md")]
    pub(crate) fn document_viewport(self) -> Rect {
        Rect::new(
            Point::new(self.viewport_scroll.0, self.viewport_scroll.1),
            self.viewport,
        )
    }

    #[doc = include_str!("surface_geometry.doc/root_transform.md")]
    pub(crate) fn root_transform(self) -> Option<AffineTransform> {
        if self.is_identity() {
            return None;
        }
        let (ox, oy) = self.content_offset;
        Some(
            AffineTransform::scale(self.device_scale, self.device_scale)
                .multiply(AffineTransform::translate(-ox, -oy)),
        )
    }

    #[doc = include_str!("surface_geometry.doc/to_content_point.md")]
    pub(crate) fn to_content_point(self, position: Point) -> Point {
        if self.is_identity() {
            return position;
        }
        let (ox, oy) = self.content_offset;
        Point::new(
            position.x / self.device_scale + ox,
            position.y / self.device_scale + oy,
        )
    }

    #[doc = include_str!("surface_geometry.doc/is_identity.md")]
    fn is_identity(self) -> bool {
        (self.device_scale - 1.0).abs() <= f32::EPSILON
            && self.content_offset.0 == 0.0
            && self.content_offset.1 == 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::SurfaceGeometry;
    use obscur_geometry::{Point, Size};

    fn surface() -> SurfaceGeometry {
        SurfaceGeometry::new(Size::new(800.0, 600.0))
    }

    #[test]
    fn an_untouched_surface_maps_nothing() {
        let s = surface();
        assert_eq!(s.device_scale(), 1.0);
        assert!(
            s.root_transform().is_none(),
            "the identity emits no transform, so the display list stays unwrapped"
        );
        let p = Point::new(37.0, 11.0);
        assert_eq!(s.to_content_point(p), p);
    }

    #[test]
    fn the_root_transform_and_the_pointer_mapping_are_inverses() {
        let mut s = surface();
        s.set_device_scale(2.0);
        s.set_content_offset(30.0, -12.5);
        let transform = s.root_transform().expect("not the identity");
        // A content point pushed through the render transform lands where the host
        // reports it, and mapping that back returns the content point.
        for content in [
            Point::new(0.0, 0.0),
            Point::new(100.0, 250.0),
            Point::new(-40.0, 7.5),
        ] {
            let host = transform.apply(content);
            let back = s.to_content_point(host);
            assert!(
                (back.x - content.x).abs() < 0.001 && (back.y - content.y).abs() < 0.001,
                "{content:?} → {host:?} → {back:?}"
            );
        }
    }

    #[test]
    fn a_scale_is_clamped_and_a_nonsense_scale_never_reaches_the_transform() {
        let mut s = surface();
        s.set_device_scale(0.0);
        assert_eq!(s.device_scale(), 0.1);
        s.set_device_scale(-4.0);
        assert_eq!(s.device_scale(), 0.1);
        s.set_device_scale(f32::NAN);
        assert_eq!(s.device_scale(), 0.1);
        assert!(s.device_scale().is_finite());
    }

    #[test]
    fn the_viewport_is_derived_from_the_clamped_scale() {
        let mut s = surface();
        // The scale is rejected and clamped to 0.1, and the viewport divides by
        // the clamp rather than by what was asked for.
        s.set_surface(Size::new(800.0, 600.0), 0.0);
        assert_eq!(s.device_scale(), 0.1);
        assert_eq!(s.viewport(), Size::new(8000.0, 6000.0));

        s.set_surface(Size::new(800.0, 600.0), 2.0);
        assert_eq!(s.viewport(), Size::new(400.0, 300.0));
    }

    #[test]
    fn only_a_change_of_asset_band_is_reported() {
        let mut s = surface();
        // 1x → 2x selects `@2x` files: a reband.
        assert!(s.set_device_scale(2.0));
        // 2.5 and 3.0 are both `@3x`: same files, no reband.
        assert!(s.set_device_scale(2.5));
        assert!(!s.set_device_scale(3.0));
    }

    #[test]
    fn a_non_finite_offset_leaves_its_axis_alone() {
        let mut s = surface();
        s.set_content_offset(10.0, 20.0);
        s.set_content_offset(f32::NAN, 50.0);
        assert_eq!(s.content_offset(), (10.0, 50.0));
        s.set_content_offset(5.0, f32::INFINITY);
        assert_eq!(s.content_offset(), (5.0, 50.0));
    }

    #[test]
    fn the_document_viewport_is_the_viewport_moved_by_its_own_scroll() {
        let mut s = surface();
        s.set_viewport_scroll(0.0, 120.0);
        let rect = s.document_viewport();
        assert_eq!(rect.origin, Point::new(0.0, 120.0));
        assert_eq!(rect.size, Size::new(800.0, 600.0));
    }
}
