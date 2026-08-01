#![doc = include_str!("surface.doc/_module.md")]

use obscur_geometry::{Insets, Size};
use obscur_style::property::EnvironmentValues;

use crate::frame::FrameReason;
use crate::runtime::EditableField;

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("surface.doc/viewport.md")]
    #[must_use]
    pub fn viewport(&self) -> Size {
        self.surface.viewport()
    }

    #[doc = include_str!("surface.doc/set_viewport.md")]
    pub fn set_viewport(&mut self, viewport: Size) {
        self.surface.set_viewport(viewport);
    }

    #[doc = include_str!("surface.doc/device_scale.md")]
    #[must_use]
    pub fn device_scale(&self) -> f32 {
        self.surface.device_scale()
    }

    #[doc = include_str!("surface.doc/content_offset.md")]
    #[must_use]
    pub fn content_offset(&self) -> (f32, f32) {
        self.surface.content_offset()
    }

    #[doc = include_str!("surface.doc/set_content_offset.md")]
    pub fn set_content_offset(&mut self, x: f32, y: f32) {
        self.surface.set_content_offset(x, y);
    }

    #[doc = include_str!("surface.doc/set_viewport_scroll.md")]
    pub fn set_viewport_scroll(&mut self, x: f32, y: f32) {
        self.surface.set_viewport_scroll(x, y);
    }

    #[doc = include_str!("surface.doc/viewport_scroll.md")]
    #[must_use]
    pub fn viewport_scroll(&self) -> (f32, f32) {
        self.surface.viewport_scroll()
    }

    #[doc = include_str!("surface.doc/environment.md")]
    #[must_use]
    pub const fn environment(&self) -> EnvironmentValues {
        self.environment
    }

    #[doc = include_str!("surface.doc/set_safe_area_insets.md")]
    pub fn set_safe_area_insets(&mut self, insets: Insets) -> bool {
        self.set_environment(self.environment.with_safe_area(insets))
    }

    #[doc = include_str!("surface.doc/set_keyboard_insets.md")]
    pub fn set_keyboard_insets(&mut self, insets: Insets) -> bool {
        self.set_environment(self.environment.with_keyboard(insets))
    }

    #[doc = include_str!("surface.doc/set_environment.md")]
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

    #[doc = include_str!("surface.doc/set_surface.md")]
    pub fn set_surface(&mut self, physical: Size, scale: f32) {
        if self.surface.set_surface(physical, scale) {
            self.invalidate_dpi_variants();
        }
    }

    #[doc = include_str!("surface.doc/set_device_scale.md")]
    pub fn set_device_scale(&mut self, scale: f32) {
        if self.surface.set_device_scale(scale) {
            self.invalidate_dpi_variants();
        }
    }

    #[doc = include_str!("surface.doc/invalidate_dpi_variants.md")]
    fn invalidate_dpi_variants(&mut self) {
        self.resources.invalidate_dpi_variants();
    }
}
