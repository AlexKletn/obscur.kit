//! [`SurfaceGeometry`]: the host surface, and the mapping between its coordinates
//! and the document's.

use obscur_geometry::{AffineTransform, Point, Rect, Size};
use obscur_interaction::gesture::ZoomPan;
use obscur_resources::source::requested_density;

/// The lowest device scale accepted. A non-positive or non-finite request lands
/// here rather than producing a degenerate or `NaN` transform.
const MIN_DEVICE_SCALE: f32 = 0.1;

#[doc = include_str!("../../docs/api/runtime/surface_geometry.doc/SurfaceGeometry.md")]
#[derive(Debug, Clone, Copy)]
pub(crate) struct SurfaceGeometry {
    /// The viewport frames are laid out under, in logical CSS pixels.
    viewport: Size,
    /// The document point pinned to the viewport's top-left, in logical pixels —
    /// the viewport's own scroll position, which is what `position: sticky`
    /// clamps against and what puts the arranged boxes into document space.
    viewport_scroll: (f32, f32),
    /// The device-pixel scale (HiDPI factor × user zoom) the composed scene
    /// renders at. Clamped to [`MIN_DEVICE_SCALE`].
    device_scale: f32,
    /// The content scroll offset `(x, y)` in logical CSS pixels — the content
    /// point pinned to the viewport top-left. A positive offset scrolls content
    /// up/left (spec §4.1).
    content_offset: (f32, f32),
    /// The optional viewport zoom and pan, separate from the device scale.
    view_camera: Option<ZoomPan>,
}

/// Reads take `self` by value and writes take `&mut self`: the type is four
/// scalars and `Copy`, so a read costs the same either way and the signature says
/// which half of the API it is in.
impl SurfaceGeometry {
    /// A surface of `viewport` logical pixels at scale `1.0`, unscrolled — the
    /// identity case, in which [`root_transform`](Self::root_transform) emits
    /// nothing and [`to_content_point`](Self::to_content_point) is the identity.
    pub(crate) const fn new(viewport: Size) -> Self {
        Self {
            viewport,
            viewport_scroll: (0.0, 0.0),
            device_scale: 1.0,
            content_offset: (0.0, 0.0),
            view_camera: None,
        }
    }

    /// The viewport frames are laid out under.
    pub(crate) const fn viewport(self) -> Size {
        self.viewport
    }

    /// Sets the viewport subsequent frames are laid out under.
    pub(crate) const fn set_viewport(&mut self, viewport: Size) {
        self.viewport = viewport;
    }

    /// The device-pixel scale the composed scene renders at.
    pub(crate) const fn device_scale(self) -> f32 {
        self.device_scale
    }

    #[doc = include_str!("../../docs/api/runtime/surface_geometry.doc/set_device_scale.md")]
    pub(crate) fn set_device_scale(&mut self, scale: f32) -> bool {
        let previous = self.device_scale;
        self.device_scale = if scale.is_finite() {
            scale.max(MIN_DEVICE_SCALE)
        } else {
            MIN_DEVICE_SCALE
        };
        requested_density(previous) != requested_density(self.device_scale)
    }

    #[doc = include_str!("../../docs/api/runtime/surface_geometry.doc/set_surface.md")]
    pub(crate) fn set_surface(&mut self, physical: Size, scale: f32) -> bool {
        let rebanded = self.set_device_scale(scale);
        let effective = self.device_scale;
        self.viewport = Size::new(physical.width / effective, physical.height / effective);
        rebanded
    }

    /// The content scroll offset `(x, y)` in logical CSS pixels.
    pub(crate) const fn content_offset(self) -> (f32, f32) {
        self.content_offset
    }

    /// Sets the content scroll offset. A non-finite component is ignored and that
    /// axis keeps its previous value.
    pub(crate) fn set_content_offset(&mut self, x: f32, y: f32) {
        let (ox, oy) = self.content_offset;
        self.content_offset = (
            if x.is_finite() { x } else { ox },
            if y.is_finite() { y } else { oy },
        );
    }

    /// The viewport's own scroll offset `(x, y)` in logical pixels.
    pub(crate) const fn viewport_scroll(self) -> (f32, f32) {
        self.viewport_scroll
    }

    /// Sets the viewport scroll offset. A non-finite component preserves the previous offset on that axis.
    pub(crate) const fn set_viewport_scroll(&mut self, x: f32, y: f32) {
        let (old_x, old_y) = self.viewport_scroll;
        self.viewport_scroll = (
            if x.is_finite() { x } else { old_x },
            if y.is_finite() { y } else { old_y },
        );
    }

    #[doc = include_str!("../../docs/api/runtime/surface_geometry.doc/document_viewport.md")]
    pub(crate) fn document_viewport(self) -> Rect {
        Rect::new(
            Point::new(self.viewport_scroll.0, self.viewport_scroll.1),
            self.viewport,
        )
    }

    /// Returns the window’s visible rectangle in content coordinates after camera zoom and pan.
    pub(crate) fn visible_content_rect(self) -> Rect {
        let (x, y) = self.content_offset;
        let camera_origin = self
            .view_camera
            .map_or(Point::ORIGIN, |camera| camera.content_point(Point::ORIGIN));
        let scale = self.camera_scale();
        Rect::new(
            Point::new(camera_origin.x + x, camera_origin.y + y),
            Size::new(self.viewport.width / scale, self.viewport.height / scale),
        )
    }

    /// Installs or clears the application’s surface camera.
    pub(crate) fn set_view_camera(&mut self, camera: Option<ZoomPan>) -> bool {
        if let Some(camera) = camera
            && (!camera.scale().recip().is_finite()
                || !camera
                    .transform()
                    .coeffs
                    .iter()
                    .all(|value| value.is_finite()))
        {
            return false;
        }
        self.view_camera = camera;
        true
    }

    /// Maps document coordinates to logical window coordinates.
    pub(crate) fn window_transform(self) -> AffineTransform {
        let (ox, oy) = self.content_offset;
        let offset = AffineTransform::translate(-ox, -oy);
        self.view_camera
            .map_or(offset, |camera| camera.transform().multiply(offset))
    }

    /// Returns the viewport camera scale independently of device scale.
    pub(crate) fn camera_scale(self) -> f32 {
        self.view_camera.map_or(1.0, |camera| camera.scale())
    }

    #[doc = include_str!("../../docs/api/runtime/surface_geometry.doc/root_transform.md")]
    pub(crate) fn root_transform(self) -> Option<AffineTransform> {
        if self.is_identity() {
            return None;
        }
        Some(
            AffineTransform::scale(self.device_scale, self.device_scale)
                .multiply(self.window_transform()),
        )
    }

    /// Maps a host `position` (physical space) into content space — the inverse of
    /// [`root_transform`](Self::root_transform), and the mapping applied before
    /// hit-testing so it happens in the space layout ran in.
    pub(crate) fn to_content_point(self, position: Point) -> Point {
        if self.is_identity() {
            return position;
        }
        let (ox, oy) = self.content_offset;
        let window = Point::new(
            position.x / self.device_scale,
            position.y / self.device_scale,
        );
        let content = self
            .view_camera
            .map_or(window, |camera| camera.content_point(window));
        Point::new(content.x + ox, content.y + oy)
    }

    /// Whether the surface maps content to host coordinates unchanged — scale
    /// exactly `1.0` and no content offset. The one condition both
    /// [`root_transform`](Self::root_transform) and
    /// [`to_content_point`](Self::to_content_point) branch on, written once so the
    /// two cannot disagree about when the mapping is trivial.
    fn is_identity(self) -> bool {
        (self.device_scale - 1.0).abs() <= f32::EPSILON && self.window_transform().is_identity()
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

    /// Mixed valid and invalid scroll coordinates update only the valid axes.
    #[test]
    fn non_finite_viewport_scroll_preserves_the_affected_axis() {
        let mut surface = surface();
        surface.set_viewport_scroll(40.0, 70.0);
        surface.set_viewport_scroll(f32::NAN, 90.0);
        assert_eq!(surface.viewport_scroll(), (40.0, 90.0));
        surface.set_viewport_scroll(-10.0, f32::INFINITY);
        assert_eq!(surface.viewport_scroll(), (-10.0, 90.0));
        surface.set_viewport_scroll(f32::NEG_INFINITY, f32::NAN);
        assert_eq!(surface.document_viewport().origin, Point::new(-10.0, 90.0));
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
