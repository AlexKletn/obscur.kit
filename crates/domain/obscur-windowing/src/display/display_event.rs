#![doc = include_str!("display_event.doc/_module.md")]

use crate::display::DisplayId;

#[doc = include_str!("display_event.doc/DisplayEvent.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplayEvent {
    #[doc = include_str!("display_event.doc/Connected.md")]
    Connected(DisplayId),
    #[doc = include_str!("display_event.doc/Disconnected.md")]
    Disconnected(DisplayId),
    #[doc = include_str!("display_event.doc/ConfigurationChanged.md")]
    ConfigurationChanged(DisplayId),
    #[doc = include_str!("display_event.doc/PrimaryChanged.md")]
    PrimaryChanged(DisplayId),
}

impl DisplayEvent {
    #[doc = include_str!("display_event.doc/display_id.md")]
    #[must_use]
    pub const fn display_id(&self) -> DisplayId {
        match self {
            Self::Connected(id)
            | Self::Disconnected(id)
            | Self::ConfigurationChanged(id)
            | Self::PrimaryChanged(id) => *id,
        }
    }

    #[doc = include_str!("display_event.doc/is_connected.md")]
    #[must_use]
    pub const fn is_connected(&self) -> bool {
        matches!(self, Self::Connected(_))
    }

    #[doc = include_str!("display_event.doc/is_disconnected.md")]
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
