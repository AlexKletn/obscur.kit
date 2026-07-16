//! The [`WindowEventConverter`] pure winit→domain window-event converter
//! (spec §6.1 windowing; §2.7 "преобразует native events в доменные события").

use obscur_geometry::{Point, ScaleFactor, Size};
use obscur_windowing::window::WindowEvent;

/// A stateless converter from winit's native window events to the domain's
/// [`WindowEvent`] (spec §6.1, §2.7).
///
/// This is a **pure** converter: it holds no event-loop state, so its output
/// depends only on its input. That determinism is the crux of the adapter's
/// correctness — the §5.2/§6.1 determinism boundary requires that identical
/// native events always yield identical domain events.
///
/// It handles only lifecycle/geometry/focus/scale/occlusion events. Input events
/// (keyboard and pointer) are routed by the dedicated
/// [`KeyboardConverter`](super::KeyboardConverter) and
/// [`PointerConverter`](super::PointerConverter); this converter maps them — and
/// any other unhandled winit event such as redraw requests — to `None`.
#[derive(Debug, Clone, Copy, Default)]
pub struct WindowEventConverter;

impl WindowEventConverter {
    /// Creates a converter.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Converts a winit window `event` to the matching domain [`WindowEvent`],
    /// or `None` when the event is routed elsewhere or ignored (spec §6.1).
    ///
    /// The numeric casts to `f32` are lossless enough at UI scale: window sizes
    /// and positions never exceed the precise integer range of `f32`, and scale
    /// factors carry only a few significant digits.
    ///
    /// # Known defect: `Resized` and `WindowProperties::size` disagree on units
    ///
    /// `Resized` and `Moved` pass winit's **physical** pixels straight through —
    /// winit types them `Resized(PhysicalSize<u32>)` (`winit-0.30.13/src/event.rs:161`)
    /// and `Moved(PhysicalPosition<i32>)`. But a window is *created* from
    /// `WindowProperties::size` in **logical** pixels:
    /// `WindowAttributes::with_inner_size(LogicalSize::new(...))`
    /// (`backend/winit_event_loop.rs:473`). Both ends are typed
    /// [`Size`](obscur_geometry::Size), so nothing in the type system flags the
    /// mismatch: a host that asks for a 480×240 window and reads back
    /// `Resized(960.0, 480.0)` on a 2× display is seeing units change under one
    /// type, not a resize.
    ///
    /// This is **not fixed here**, and no test pins it — a test asserting the
    /// current behaviour would be asserting the bug. It is invisible at scale 1,
    /// which is why it survived: the physical passthrough is what a wgpu
    /// swapchain wants, so the reference host in `windowed_counter` is correct
    /// today *by coincidence of scale*, not by contract. Deciding it properly
    /// means choosing whether the domain `Size` is logical (and dividing here,
    /// which needs the per-window scale factor this converter is not given) or
    /// physical (and converting at creation instead) — a domain-semantics call
    /// beyond this converter, and one that wants a HiDPI display to verify.
    #[must_use]
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    pub fn convert(&self, event: &winit::event::WindowEvent) -> Option<WindowEvent> {
        use winit::event::WindowEvent as Winit;
        match event {
            Winit::Resized(size) => Some(WindowEvent::Resized(Size::new(
                size.width as f32,
                size.height as f32,
            ))),
            Winit::Moved(position) => Some(WindowEvent::Moved(Point::new(
                position.x as f32,
                position.y as f32,
            ))),
            Winit::CloseRequested => Some(WindowEvent::CloseRequested),
            Winit::Destroyed => Some(WindowEvent::Destroyed),
            Winit::Focused(true) => Some(WindowEvent::FocusGained),
            Winit::Focused(false) => Some(WindowEvent::FocusLost),
            Winit::Occluded(occluded) => Some(WindowEvent::Occluded(*occluded)),
            Winit::ScaleFactorChanged { scale_factor, .. } => Some(
                WindowEvent::ScaleFactorChanged(ScaleFactor::new(*scale_factor as f32)),
            ),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::dpi::{PhysicalPosition, PhysicalSize};
    use winit::event::WindowEvent as Winit;

    #[test]
    fn resized_maps_to_domain_size() {
        let converter = WindowEventConverter::new();
        let event = Winit::Resized(PhysicalSize::new(800, 600));
        assert_eq!(
            converter.convert(&event),
            Some(WindowEvent::Resized(Size::new(800.0, 600.0)))
        );
    }

    #[test]
    fn moved_maps_to_domain_point() {
        let converter = WindowEventConverter::new();
        let event = Winit::Moved(PhysicalPosition::new(10, 20));
        assert_eq!(
            converter.convert(&event),
            Some(WindowEvent::Moved(Point::new(10.0, 20.0)))
        );
    }

    #[test]
    fn close_requested_and_destroyed_map_across() {
        let converter = WindowEventConverter::new();
        assert_eq!(
            converter.convert(&Winit::CloseRequested),
            Some(WindowEvent::CloseRequested)
        );
        assert_eq!(
            converter.convert(&Winit::Destroyed),
            Some(WindowEvent::Destroyed)
        );
    }

    #[test]
    fn focus_maps_to_gained_and_lost() {
        let converter = WindowEventConverter::new();
        assert_eq!(
            converter.convert(&Winit::Focused(true)),
            Some(WindowEvent::FocusGained)
        );
        assert_eq!(
            converter.convert(&Winit::Focused(false)),
            Some(WindowEvent::FocusLost)
        );
    }

    #[test]
    fn occluded_carries_the_flag() {
        let converter = WindowEventConverter::new();
        assert_eq!(
            converter.convert(&Winit::Occluded(true)),
            Some(WindowEvent::Occluded(true))
        );
        assert_eq!(
            converter.convert(&Winit::Occluded(false)),
            Some(WindowEvent::Occluded(false))
        );
    }
}
