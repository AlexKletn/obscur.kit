#![doc = include_str!("window_event_converter.doc/_module.md")]

use obscur_geometry::{Point, ScaleFactor, Size};
use obscur_windowing::window::WindowEvent;

#[doc = include_str!("window_event_converter.doc/WindowEventConverter.md")]
#[derive(Debug, Clone, Copy, Default)]
pub struct WindowEventConverter;

impl WindowEventConverter {
    #[doc = include_str!("window_event_converter.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("window_event_converter.doc/convert.md")]
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
