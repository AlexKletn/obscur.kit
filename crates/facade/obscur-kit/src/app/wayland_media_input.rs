//! Maps Wayland seat capabilities to CSS media input for window and layer runners.

use obscur_wayland_effects::WaylandPointingCapabilities;

use crate::style::matching::{MediaInput, PointerAccuracy};

/// Maps the seat pointer to a fine hovering device and touch to a coarse device. The pointer is primary when both are available.
pub(super) fn media_input_from_seats(capabilities: WaylandPointingCapabilities) -> MediaInput {
    let primary = if capabilities.pointer {
        Some(PointerAccuracy::Fine)
    } else if capabilities.touch {
        Some(PointerAccuracy::Coarse)
    } else {
        None
    };
    MediaInput {
        primary,
        primary_hover: capabilities.pointer,
        additional_coarse: capabilities.pointer && capabilities.touch,
        ..MediaInput::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seat_pointer_is_primary_when_touch_is_also_available() {
        let input = media_input_from_seats(WaylandPointingCapabilities {
            pointer: true,
            touch: true,
        });
        assert_eq!(input.primary, Some(PointerAccuracy::Fine));
        assert!(input.primary_hover);
        assert!(input.additional_coarse);

        let touch_only = media_input_from_seats(WaylandPointingCapabilities {
            pointer: false,
            touch: true,
        });
        assert_eq!(touch_only.primary, Some(PointerAccuracy::Coarse));
        assert!(!touch_only.primary_hover);
        assert!(!touch_only.additional_coarse);
    }
}
