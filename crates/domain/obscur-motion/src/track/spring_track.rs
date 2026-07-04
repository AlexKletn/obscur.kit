//! The [`SpringTrack`] spring-shaped from→to track (spec §4.17).

use crate::track::MotionTrack;
use crate::value::MotionValue;

/// A `from` → `to` motion shaped by a normalized spring curve (spec §4.17).
///
/// This is the native compile target of a spring `animation`: rather than a
/// fixed easing table, the progress is the analytic displacement of a damped
/// harmonic oscillator (`m·x'' + c·x' + k·x = 0`) mapped onto `t ∈ [0, 1]`. A
/// *full* velocity-carrying physical spring — one that accepts an initial
/// velocity and integrates real time to rest — is a driver-level concern; here
/// the spring is a pure, deterministic **normalized curve** with `f(0) == 0` and
/// `f(1) ≈ 1`.
///
/// Springs overshoot: for an underdamped configuration the progress can exceed
/// `1.0` (and dip back) before settling. That is intentional and desirable — the
/// underlying [`MotionValue`] interpolation does not clamp, so the sampled value
/// overshoots its target the way a physical spring would.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringTrack {
    /// The value at rest at the start (`t == 0`).
    from: MotionValue,
    /// The target the spring settles toward (`t ≈ 1`).
    to: MotionValue,
    /// Spring constant `k`: higher is stiffer / faster.
    stiffness: f32,
    /// Damping coefficient `c`: higher settles sooner with less overshoot.
    damping: f32,
    /// Oscillator mass `m`: higher is heavier / slower.
    mass: f32,
}

impl SpringTrack {
    /// Creates a spring from `from` to `to` with common UI defaults
    /// (stiffness `170`, damping `26`, mass `1`).
    #[must_use]
    pub const fn new(from: MotionValue, to: MotionValue) -> Self {
        Self {
            from,
            to,
            stiffness: 170.0,
            damping: 26.0,
            mass: 1.0,
        }
    }

    /// Returns this spring with `stiffness` set (fluent setter).
    #[must_use]
    pub const fn with_stiffness(mut self, stiffness: f32) -> Self {
        self.stiffness = stiffness;
        self
    }

    /// Returns this spring with `damping` set (fluent setter).
    #[must_use]
    pub const fn with_damping(mut self, damping: f32) -> Self {
        self.damping = damping;
        self
    }

    /// Returns this spring with `mass` set (fluent setter).
    #[must_use]
    pub const fn with_mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }

    /// Returns the start (`t == 0`) value.
    #[must_use]
    pub const fn from(&self) -> MotionValue {
        self.from
    }

    /// Returns the target value.
    #[must_use]
    pub const fn to(&self) -> MotionValue {
        self.to
    }

    /// Returns the spring constant `k`.
    #[must_use]
    pub const fn stiffness(&self) -> f32 {
        self.stiffness
    }

    /// Returns the damping coefficient `c`.
    #[must_use]
    pub const fn damping(&self) -> f32 {
        self.damping
    }

    /// Returns the oscillator mass `m`.
    #[must_use]
    pub const fn mass(&self) -> f32 {
        self.mass
    }

    /// Computes normalized spring displacement in `[0, ~1]` at progress `t`.
    ///
    /// The analytic solution of `m·x'' + c·x' + k·x = 0` gives the *remaining*
    /// displacement `r(t)`; progress is `1 − r(t)`. Time is `T = t · 6` so the
    /// spring mostly settles within `t ∈ [0, 1]`. `f(0)` is exactly `0`;
    /// underdamped springs overshoot past `1` before settling.
    fn spring_progress(&self, t: f32) -> f32 {
        if t <= 0.0 {
            return 0.0;
        }
        let time = t * 6.0;
        let omega0 = (self.stiffness / self.mass).sqrt();
        let zeta = self.damping / (2.0 * (self.stiffness * self.mass).sqrt());
        let remaining = if zeta < 1.0 {
            // Underdamped: oscillatory decay toward the target.
            let omega_d = omega0 * (1.0 - zeta * zeta).sqrt();
            (-zeta * omega0 * time).exp()
                * ((omega_d * time).cos() + (zeta * omega0 / omega_d) * (omega_d * time).sin())
        } else {
            // Critically/overdamped: monotone decay toward the target.
            (-omega0 * time).exp() * (1.0 + omega0 * time)
        };
        1.0 - remaining
    }
}

impl MotionTrack for SpringTrack {
    /// Samples the spring blend from `from` toward `to` at progress `t`.
    ///
    /// `t` is passed through the spring curve *without* clamping so overshoot is
    /// preserved; `t <= 0` yields exactly `from`. On a variant mismatch between
    /// the endpoints the blend is undefined and this falls back to `to`.
    fn sample(&self, t: f32) -> MotionValue {
        self.from
            .interpolate_checked(&self.to, self.spring_progress(t))
            .unwrap_or(self.to)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::Point;

    #[test]
    fn defaults_are_common_ui_spring() {
        let spring = SpringTrack::new(MotionValue::scalar(0.0), MotionValue::scalar(1.0));
        assert_eq!(spring.stiffness(), 170.0);
        assert_eq!(spring.damping(), 26.0);
        assert_eq!(spring.mass(), 1.0);
        assert_eq!(spring.from(), MotionValue::scalar(0.0));
        assert_eq!(spring.to(), MotionValue::scalar(1.0));
    }

    #[test]
    fn progress_is_zero_at_start() {
        let spring = SpringTrack::new(MotionValue::scalar(0.0), MotionValue::scalar(1.0));
        assert_eq!(spring.spring_progress(0.0), 0.0);
        assert_eq!(spring.spring_progress(-1.0), 0.0);
    }

    #[test]
    fn sample_from_endpoint_is_exact() {
        let spring = SpringTrack::new(MotionValue::scalar(3.0), MotionValue::scalar(9.0));
        assert_eq!(spring.sample(0.0), MotionValue::scalar(3.0));
    }

    #[test]
    fn settles_near_target_by_end() {
        let spring = SpringTrack::new(MotionValue::scalar(0.0), MotionValue::scalar(1.0));
        // By t = 1 (and beyond) the underdamped default spring has settled to ~1.
        let mut i: i16 = 100;
        while i <= 200 {
            let t = f32::from(i) / 100.0;
            let progress = spring.spring_progress(t);
            assert!(
                (progress - 1.0).abs() < 0.05,
                "spring_progress({t}) = {progress} not near 1"
            );
            i += 1;
        }
    }

    #[test]
    fn overdamped_branch_is_monotone_and_settles() {
        // Heavy damping selects the critically/overdamped branch (zeta >= 1).
        let spring = SpringTrack::new(MotionValue::scalar(0.0), MotionValue::scalar(1.0))
            .with_stiffness(100.0)
            .with_damping(100.0)
            .with_mass(1.0);
        assert_eq!(spring.spring_progress(0.0), 0.0);
        assert!((spring.spring_progress(1.0) - 1.0).abs() < 0.05);
    }

    #[test]
    fn type_mismatch_falls_back_to_to() {
        let to = MotionValue::point(Point::new(1.0, 2.0));
        let spring = SpringTrack::new(MotionValue::scalar(0.0), to);
        assert_eq!(spring.sample(0.5), to);
    }

    #[test]
    fn is_object_safe() {
        let spring: Box<dyn MotionTrack> = Box::new(SpringTrack::new(
            MotionValue::scalar(0.0),
            MotionValue::scalar(1.0),
        ));
        assert_eq!(spring.sample(0.0), MotionValue::scalar(0.0));
    }
}
