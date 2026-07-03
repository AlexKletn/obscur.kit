//! The [`MotionTrack`] track port and the shared [`Easing`] helper (spec §4.17).
//!
//! A *track* turns a normalized timeline progress `t` into a presentation
//! [`MotionValue`]. The timeline upstream produces only raw linear progress;
//! tracks are where easing curves and value interpolation are applied, so the
//! track is the compile target for CSS `transition` / `@keyframes` / spring
//! `animation` once the timeline has been separated out. [`Easing`] lives here
//! because it is shared by both the transition and keyframe tracks.

use crate::value::MotionValue;

/// A source of presentation values sampled by normalized progress (spec §4.17).
///
/// Every track — [`TransitionTrack`](super::TransitionTrack),
/// [`KeyframeTrack`](super::KeyframeTrack),
/// [`SpringTrack`](super::SpringTrack) — implements this one method: given a
/// normalized progress `t`, produce the [`MotionValue`] to present. The trait is
/// deliberately **object-safe** (no generics, no `Self`-returning methods) so a
/// heterogeneous set of tracks can be held as `Box<dyn MotionTrack>`.
///
/// `t` is normalized progress in `[0, 1]`. Implementations that shape `t`
/// through a bounded easing curve clamp it to `[0, 1]` first (documented on each
/// impl); spring-shaped tracks intentionally do *not* clamp, because a spring
/// overshoots past its target before settling.
pub trait MotionTrack {
    /// Samples the presentation value at normalized progress `t` (spec §4.17).
    ///
    /// `t == 0.0` yields the start value and `t == 1.0` the end value.
    /// Bounded-easing tracks clamp `t` into `[0, 1]`; spring tracks pass it
    /// through so overshoot is preserved. Pure and deterministic: sampling the
    /// same track at the same `t` always yields the same value.
    #[must_use]
    fn sample(&self, t: f32) -> MotionValue;
}

/// A bounded easing curve applied to normalized progress (spec §4.17).
///
/// These are the native compile targets of the common CSS timing functions:
/// `linear`, `ease-in`, `ease-out` and `ease-in-out`. Each maps normalized
/// progress `t ∈ [0, 1]` to eased progress in `[0, 1]` with `apply(0) == 0` and
/// `apply(1) == 1`. Full arbitrary `cubic-bezier(...)` / `steps(...)` timing
/// functions are an *adapter* concern: a compiler lowers them onto this enum (or
/// a keyframe sampling of the curve) rather than the domain modeling every
/// bezier directly.
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
    /// Maps normalized progress `t` through this easing curve (spec §4.17).
    ///
    /// `t` is clamped to `[0, 1]` first, so every variant satisfies
    /// `apply(0) == 0` and `apply(1) == 1` and stays within `[0, 1]` in between.
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
