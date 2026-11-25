#![doc = include_str!("../../docs/api/pointer/wheel_delta_mode.doc/_module.md")]

/// The unit a `WheelEvent`'s deltas are expressed in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WheelDeltaMode {
    #[doc = include_str!("../../docs/api/pointer/wheel_delta_mode.doc/Pixels.md")]
    #[default]
    Pixels,
    #[doc = include_str!("../../docs/api/pointer/wheel_delta_mode.doc/Lines.md")]
    Lines,
}

impl WheelDeltaMode {
    /// The delta in pixels, given what a line is worth.
    #[must_use]
    pub fn resolve(self, delta: f32, line_height: f32) -> f32 {
        match self {
            Self::Pixels => delta,
            Self::Lines => delta * line_height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WheelDeltaMode;

    #[test]
    fn pixels_are_already_pixels() {
        assert_eq!(WheelDeltaMode::Pixels.resolve(-25.0, 48.0), -25.0);
    }

    #[test]
    fn lines_are_multiplied_by_the_line_height() {
        assert_eq!(WheelDeltaMode::Lines.resolve(-3.0, 48.0), -144.0);
    }

    #[test]
    fn the_sign_survives_either_way() {
        // The sign is the whole meaning of a delta and no conversion may eat it.
        assert!(WheelDeltaMode::Lines.resolve(-1.0, 48.0) < 0.0);
        assert!(WheelDeltaMode::Lines.resolve(1.0, 48.0) > 0.0);
    }

    #[test]
    fn pixels_is_the_default_because_it_needs_no_policy() {
        // A host that says nothing is taken to have reported pixels, which is
        // the reading that does not silently multiply anything by 48.
        assert_eq!(WheelDeltaMode::default(), WheelDeltaMode::Pixels);
    }
}
