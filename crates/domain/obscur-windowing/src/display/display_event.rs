//! Display hotplug and configuration events (spec §6.6).

use crate::display::DisplayId;

#[doc = include_str!("../../docs/api/display/display_event.doc/DisplayEvent.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplayEvent {
    /// A display was connected and is now available.
    Connected(DisplayId),
    /// A display was disconnected and is no longer available.
    Disconnected(DisplayId),
    /// A display's configuration changed (mode, scale, or bounds).
    ConfigurationChanged(DisplayId),
    /// A display became the primary display.
    PrimaryChanged(DisplayId),
}

impl DisplayEvent {
    /// Returns the identifier of the display this event concerns.
    #[must_use]
    pub const fn display_id(&self) -> DisplayId {
        match self {
            Self::Connected(id)
            | Self::Disconnected(id)
            | Self::ConfigurationChanged(id)
            | Self::PrimaryChanged(id) => *id,
        }
    }

    /// Returns whether this event reports a display being connected.
    #[must_use]
    pub const fn is_connected(&self) -> bool {
        matches!(self, Self::Connected(_))
    }

    /// Returns whether this event reports a display being disconnected.
    #[must_use]
    pub const fn is_disconnected(&self) -> bool {
        matches!(self, Self::Disconnected(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_id_across_all_variants() {
        let id = DisplayId::from_raw(9);
        assert_eq!(DisplayEvent::Connected(id).display_id(), id);
        assert_eq!(DisplayEvent::Disconnected(id).display_id(), id);
        assert_eq!(DisplayEvent::ConfigurationChanged(id).display_id(), id);
        assert_eq!(DisplayEvent::PrimaryChanged(id).display_id(), id);
    }

    #[test]
    fn connection_predicates() {
        let id = DisplayId::from_raw(1);
        assert!(DisplayEvent::Connected(id).is_connected());
        assert!(!DisplayEvent::Connected(id).is_disconnected());
        assert!(DisplayEvent::Disconnected(id).is_disconnected());
        assert!(!DisplayEvent::Disconnected(id).is_connected());
        assert!(!DisplayEvent::ConfigurationChanged(id).is_connected());
    }
}
