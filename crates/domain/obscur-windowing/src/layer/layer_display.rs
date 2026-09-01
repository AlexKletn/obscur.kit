#![doc = include_str!("layer_display.doc/_module.md")]

use crate::display::DisplayId;

#[doc = include_str!("layer_display.doc/LayerDisplay.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LayerDisplay {
    id: DisplayId,
    name: String,
}

impl LayerDisplay {
    #[doc = include_str!("layer_display.doc/new.md")]
    #[must_use]
    pub fn new(id: DisplayId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
        }
    }

    #[doc = include_str!("layer_display.doc/id.md")]
    #[must_use]
    pub const fn id(&self) -> DisplayId {
        self.id
    }

    #[doc = include_str!("layer_display.doc/name.md")]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_display_carries_the_id_settings_take_and_the_name_a_person_reads() {
        let display = LayerDisplay::new(DisplayId::from_raw(2), "DP-3");
        assert_eq!(display.id(), DisplayId::from_raw(2));
        assert_eq!(display.name(), "DP-3");
    }

    #[test]
    fn two_displays_are_told_apart_by_id_not_by_name() {
        // Duplicate connector names are rare but not impossible, and the id is
        // what `LayerSettings::with_display` actually carries.
        let left = LayerDisplay::new(DisplayId::from_raw(1), "HDMI-A-1");
        let right = LayerDisplay::new(DisplayId::from_raw(2), "HDMI-A-1");
        assert_ne!(left, right);
        assert_eq!(left.name(), right.name());
    }
}
