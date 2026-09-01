#![doc = include_str!("layer_surface_event.doc/_module.md")]

use obscur_geometry::{ScaleFactor, Size};

#[doc = include_str!("layer_surface_event.doc/LayerSurfaceEvent.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayerSurfaceEvent {
    #[doc = include_str!("layer_surface_event.doc/Configured.md")]
    Configured(Size),
    #[doc = include_str!("layer_surface_event.doc/Resized.md")]
    Resized(Size),
    #[doc = include_str!("layer_surface_event.doc/ScaleFactorChanged.md")]
    ScaleFactorChanged(ScaleFactor),
    #[doc = include_str!("layer_surface_event.doc/CloseRequested.md")]
    CloseRequested,
    #[doc = include_str!("layer_surface_event.doc/Woken.md")]
    Woken,
}

impl LayerSurfaceEvent {
    #[doc = include_str!("layer_surface_event.doc/new_size.md")]
    #[must_use]
    pub const fn new_size(&self) -> Option<Size> {
        match self {
            Self::Configured(size) | Self::Resized(size) => Some(*size),
            _ => None,
        }
    }

    #[doc = include_str!("layer_surface_event.doc/redraws.md")]
    #[must_use]
    pub const fn redraws(&self) -> bool {
        matches!(
            self,
            Self::Configured(_) | Self::Resized(_) | Self::ScaleFactorChanged(_) | Self::Woken
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_events_that_carry_geometry_answer_with_it() {
        let size = Size::new(1920.0, 36.0);
        assert_eq!(LayerSurfaceEvent::Configured(size).new_size(), Some(size));
        assert_eq!(LayerSurfaceEvent::Resized(size).new_size(), Some(size));
        assert_eq!(LayerSurfaceEvent::Woken.new_size(), None);
        assert_eq!(LayerSurfaceEvent::CloseRequested.new_size(), None);
    }

    #[test]
    fn the_only_event_that_does_not_call_for_a_frame_is_the_last_one() {
        for event in [
            LayerSurfaceEvent::Configured(Size::new(100.0, 20.0)),
            LayerSurfaceEvent::Resized(Size::new(100.0, 20.0)),
            LayerSurfaceEvent::ScaleFactorChanged(ScaleFactor::new(2.0)),
            LayerSurfaceEvent::Woken,
        ] {
            assert!(event.redraws(), "{event:?} leaves the surface stale");
        }
        assert!(!LayerSurfaceEvent::CloseRequested.redraws());
    }

    #[test]
    fn a_wake_carries_no_reason() {
        // Deliberate: the waker means "reconsider the interface" and nothing
        // finer, so there is no payload for a caller to branch on wrongly.
        assert_eq!(LayerSurfaceEvent::Woken.new_size(), None);
        assert!(LayerSurfaceEvent::Woken.redraws());
    }
}
