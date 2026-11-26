//! Touch resize for undecorated Wayland windows whose backend only exposes mouse grabs.

use winit::{
    dpi::{LogicalPosition, LogicalSize},
    event::{Touch, TouchPhase},
    window::Window,
};

use super::{AppRunner, WindowSizeLimits};

const CORNER_TARGET: f64 = 24.0;

pub(super) struct TouchWindowResize {
    id: u64,
    origin: LogicalPosition<f64>,
    size: LogicalSize<f64>,
}

impl TouchWindowResize {
    fn begin(id: u64, origin: LogicalPosition<f64>, size: LogicalSize<f64>) -> Option<Self> {
        let right = size.width - origin.x;
        let bottom = size.height - origin.y;
        let within = |distance| (0.0..=CORNER_TARGET).contains(&distance);
        (within(right) && within(bottom)).then_some(Self { id, origin, size })
    }

    fn size_at(&self, point: LogicalPosition<f64>, limits: WindowSizeLimits) -> LogicalSize<f64> {
        let (min_width, min_height) = limits.minimum.unwrap_or((1.0, 1.0));
        let (max_width, max_height) = limits.maximum.unwrap_or((f32::INFINITY, f32::INFINITY));
        LogicalSize::new(
            (self.size.width + point.x - self.origin.x)
                .clamp(f64::from(min_width), f64::from(max_width)),
            (self.size.height + point.y - self.origin.y)
                .clamp(f64::from(min_height), f64::from(max_height)),
        )
    }
}

impl AppRunner {
    pub(super) fn handle_touch_window_resize(&mut self, contact: Touch) -> bool {
        let Some(window) = self.window.as_ref() else {
            return false;
        };
        let point = contact.location.to_logical(window.scale_factor());
        let resizable = self.resize
            && !self.decorated
            && !window.is_maximized()
            && window.fullscreen().is_none();

        if let Some(gesture) = &self.touch_window_resize {
            if gesture.id != contact.id {
                return false;
            }
            match contact.phase {
                TouchPhase::Moved if resizable => {
                    let previous = window.inner_size();
                    if let Some(size) =
                        window.request_inner_size(gesture.size_at(point, self.size_limits))
                        && size != previous
                    {
                        self.accept_window_extent((size.width, size.height));
                        self.sync_window_state();
                    }
                }
                TouchPhase::Ended | TouchPhase::Cancelled => self.touch_window_resize = None,
                _ => {}
            }
            return true;
        }

        if resizable && contact.phase == TouchPhase::Started {
            self.touch_window_resize =
                TouchWindowResize::begin(contact.id, point, logical_size(window));
        }
        self.touch_window_resize.is_some()
    }
}

fn logical_size(window: &Window) -> LogicalSize<f64> {
    window.inner_size().to_logical(window.scale_factor())
}

#[cfg(test)]
#[path = "touch_window_resize.tests.rs"]
mod tests;
