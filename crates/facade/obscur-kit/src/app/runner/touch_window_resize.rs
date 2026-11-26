use winit::event::{Touch, TouchPhase};

use super::{AppRunner, window_resize::resize_grip};

impl AppRunner {
    pub(super) fn handle_touch_window_resize(&mut self, contact: Touch) -> bool {
        if contact.phase != TouchPhase::Started || !self.resize || self.decorated {
            return false;
        }
        let Some(window) = self.window.as_ref() else {
            return false;
        };
        if window.is_maximized() || window.fullscreen().is_some() {
            return false;
        }
        let point = contact.location.to_logical::<f32>(window.scale_factor());
        let Some(direction) = resize_grip(
            self.runtime.viewport(),
            self.runtime.viewport_corner_radii(),
            crate::geometry::Point::new(point.x, point.y),
            10.0,
            Some(24.0),
        ) else {
            return false;
        };
        let button = crate::ui_ir::name_binding::component_id_for(
            "button",
            crate::ui_ir::name_binding::TagPosition::Element,
        );
        let physical = contact.location.cast::<f32>();
        if self
            .runtime
            .hit_test(crate::geometry::Point::new(physical.x, physical.y))
            .nodes()
            .into_iter()
            .filter_map(|id| self.runtime.tree().get(id))
            .any(|node| node.kind().component_id() == Some(button))
        {
            return false;
        }
        self.wayland_touch_move.as_mut().is_some_and(|native| {
            match native.resize(
                contact.id,
                super::window_actions::wayland_resize_edge(direction),
            ) {
                Ok(started) => started,
                Err(error) => {
                    eprintln!("Wayland touch resize failed: {error}");
                    false
                }
            }
        })
    }
}
