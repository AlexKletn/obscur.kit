#[doc = include_str!("../../docs/api/track/motion_track.doc/Easing.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Easing {
    /// Constant rate: `apply(t) == t` (CSS `linear`).
    #[default]
    Linear,
    /// Accelerating from zero: `t²` (CSS `ease-in`).
    EaseIn,
    /// Decelerating to the end: `t·(2 − t)` (CSS `ease-out`).
    EaseOut,
    /// Accelerate then decelerate: the standard piecewise curve (CSS
    /// `ease-in-out`).
    EaseInOut,
}

impl Easing {
    #[doc = include_str!("../../docs/api/track/motion_track.doc/apply.md")]
    #[must_use]
    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::EaseIn => t * t,
            Self::EaseOut => t * (2.0 - t),
            Self::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    let f = -2.0 * t + 2.0;
                    1.0 - (f * f) / 2.0
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Easing; 4] = [
        Easing::Linear,
        Easing::EaseIn,
        Easing::EaseOut,
        Easing::EaseInOut,
    ];

    #[test]
    fn default_is_linear() {
        assert_eq!(Easing::default(), Easing::Linear);
    }

    #[test]
    fn endpoints_are_fixed_for_every_variant() {
        for easing in ALL {
            assert_eq!(easing.apply(0.0), 0.0, "{easing:?} apply(0)");
            assert_eq!(easing.apply(1.0), 1.0, "{easing:?} apply(1)");
        }
    }

    #[test]
    fn out_of_range_input_is_clamped() {
        for easing in ALL {
            assert_eq!(easing.apply(-1.0), 0.0, "{easing:?} apply(-1)");
            assert_eq!(easing.apply(2.0), 1.0, "{easing:?} apply(2)");
        }
    }

    #[test]
    fn midpoints_are_monotonic_and_within_unit_range() {
        // Every curve is non-decreasing across a fine sweep and stays in [0, 1].
        for easing in ALL {
            let mut prev = 0.0;
            let mut i: i16 = 0;
            while i <= 100 {
                let t = f32::from(i) / 100.0;
                let v = easing.apply(t);
                assert!((0.0..=1.0).contains(&v), "{easing:?} at {t} -> {v}");
                assert!(v >= prev - 1e-6, "{easing:?} not monotonic at {t}");
                prev = v;
                i += 1;
            }
        }
    }

    #[test]
    fn known_midpoint_values() {
        assert_eq!(Easing::Linear.apply(0.5), 0.5);
        assert_eq!(Easing::EaseIn.apply(0.5), 0.25);
        assert_eq!(Easing::EaseOut.apply(0.5), 0.75);
        assert_eq!(Easing::EaseInOut.apply(0.5), 0.5);
    }
}
