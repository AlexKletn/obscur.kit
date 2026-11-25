#![doc = include_str!("../../../docs/api/runtime/ui_runtime/surface.doc/_module.md")]

use obscur_geometry::{Insets, Rect, Size};
use obscur_interaction::gesture::ZoomPan;
use obscur_style::property::EnvironmentValues;

use crate::frame::FrameReason;
use crate::runtime::EditableField;

use super::UiRuntime;

impl UiRuntime {
    /// Returns the viewport frames are laid out under.
    #[must_use]
    pub fn viewport(&self) -> Size {
        self.surface.viewport()
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/surface.doc/set_viewport.md")]
    pub fn set_viewport(&mut self, viewport: Size) {
        self.surface.set_viewport(viewport);
    }

    /// Returns the current device-pixel scale (HiDPI factor × user zoom) the
    /// composed scene renders at. Default `1.0`.
    #[must_use]
    pub fn device_scale(&self) -> f32 {
        self.surface.device_scale()
    }

    /// Returns the current content scroll offset `(x, y)` in logical CSS pixels.
    /// Default `(0.0, 0.0)`.
    #[must_use]
    pub fn content_offset(&self) -> (f32, f32) {
        self.surface.content_offset()
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/surface.doc/set_content_offset.md")]
    pub fn set_content_offset(&mut self, x: f32, y: f32) {
        self.surface.set_content_offset(x, y);
    }

    /// Reports the rectangle visible through the current window camera in logical content coordinates.
    #[must_use]
    pub fn visible_content_rect(&self) -> Rect {
        self.surface.visible_content_rect()
    }

    /// Applies an application-managed zoom and pan to the whole surface in logical window coordinates. The camera changes rendering and pointer mapping while retaining layout and device scale. Pass None to clear the camera.
    pub fn set_view_camera(&mut self, camera: Option<ZoomPan>) -> bool {
        if !self.surface.set_view_camera(camera) {
            return false;
        }
        self.request_frame(FrameReason::Input);
        true
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/surface.doc/set_viewport_scroll.md")]
    pub fn set_viewport_scroll(&mut self, x: f32, y: f32) {
        self.surface.set_viewport_scroll(x, y);
    }

    /// Returns the current viewport scroll offset `(x, y)` in logical pixels.
    #[must_use]
    pub fn viewport_scroll(&self) -> (f32, f32) {
        self.surface.viewport_scroll()
    }

    /// The environment values currently in force (spec §6.15).
    #[must_use]
    pub const fn environment(&self) -> EnvironmentValues {
        self.environment
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/surface.doc/set_safe_area_insets.md")]
    pub fn set_safe_area_insets(&mut self, insets: Insets) -> bool {
        self.set_environment(self.environment.with_safe_area(insets))
    }

    /// Reports the region the on-screen keyboard occludes, as insets from each
    /// viewport edge; all zeroes when it is hidden (spec §6.15). Returns whether
    /// anything changed.
    pub fn set_keyboard_insets(&mut self, insets: Insets) -> bool {
        self.set_environment(self.environment.with_keyboard(insets))
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/surface.doc/set_environment.md")]
    pub fn set_environment(&mut self, environment: EnvironmentValues) -> bool {
        if self.environment == environment {
            return false;
        }
        let keyboard_moved = self.environment.keyboard != environment.keyboard;
        self.environment = environment;
        // Every `env()` in the sheet computes to a different length now, and a
        // computed length is what layout arranges from — so the next frame must
        // re-resolve rather than reuse. The retained-style memo is keyed on the
        // environment, so it declines by itself; this only has to ask for the
        // frame that will notice.
        self.request_frame(FrameReason::WindowChange);
        // §6.15 "Focused inputs may be scrolled into view". A keyboard that has
        // just risen over the field the user is typing in is the reason that
        // sentence is in the spec: nothing about the *focus* changed, so only the
        // keyboard's arrival can trigger the reveal.
        if keyboard_moved
            && let Some(focused) = self.focused
            && let Some(field) = EditableField::editable_ancestor(&self.tree, focused)
        {
            self.scroll_into_view(field);
        }
        true
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/surface.doc/set_surface.md")]
    pub fn set_surface(&mut self, physical: Size, scale: f32) {
        let previous = self.device_scale();
        if self.surface.set_surface(physical, scale) {
            self.invalidate_dpi_variants();
        }
        self.invalidate_resolution_queries(previous);
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/surface.doc/set_device_scale.md")]
    pub fn set_device_scale(&mut self, scale: f32) {
        let previous = self.device_scale();
        if self.surface.set_device_scale(scale) {
            self.invalidate_dpi_variants();
        }
        self.invalidate_resolution_queries(previous);
    }

    /// Invalidates the style cache on scale changes even when the resource density band remains the same.
    fn invalidate_resolution_queries(&mut self, previous: f32) {
        if previous != self.device_scale() {
            self.last_style = None;
            self.request_frame(FrameReason::WindowChange);
        }
    }

    /// Drops every resident decoded image so the next frame's resolve pass loads it
    /// again against the current device scale's DPI-variant ladder (spec §5.16).
    fn invalidate_dpi_variants(&mut self) {
        self.resources.invalidate_dpi_variants();
    }
}
