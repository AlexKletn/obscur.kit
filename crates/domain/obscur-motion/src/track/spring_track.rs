#![doc = include_str!("spring_track.doc/_module.md")]

use crate::track::MotionTrack;
use crate::value::MotionValue;

#[doc = include_str!("spring_track.doc/SpringTrack.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringTrack {
    #[doc = include_str!("spring_track.doc/from.md")]
    from: MotionValue,
    #[doc = include_str!("spring_track.doc/to.md")]
    to: MotionValue,
    #[doc = include_str!("spring_track.doc/stiffness.md")]
    stiffness: f32,
    #[doc = include_str!("spring_track.doc/damping.md")]
    damping: f32,
    #[doc = include_str!("spring_track.doc/mass.md")]
    mass: f32,
}

impl SpringTrack {
    #[doc = include_str!("spring_track.doc/new.md")]
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

    #[doc = include_str!("spring_track.doc/with_stiffness.md")]
    #[must_use]
    pub const fn with_stiffness(mut self, stiffness: f32) -> Self {
        self.stiffness = stiffness;
        self
    }

    #[doc = include_str!("spring_track.doc/with_damping.md")]
    #[must_use]
    pub const fn with_damping(mut self, damping: f32) -> Self {
        self.damping = damping;
        self
    }

    #[doc = include_str!("spring_track.doc/with_mass.md")]
    #[must_use]
    pub const fn with_mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }

    #[doc = include_str!("spring_track.doc/from_2.md")]
    #[must_use]
    pub const fn from(&self) -> MotionValue {
        self.from
    }

    #[doc = include_str!("spring_track.doc/to_2.md")]
    #[must_use]
    pub const fn to(&self) -> MotionValue {
        self.to
    }

    #[doc = include_str!("spring_track.doc/stiffness_2.md")]
    #[must_use]
    pub const fn stiffness(&self) -> f32 {
        self.stiffness
    }

    #[doc = include_str!("spring_track.doc/damping_2.md")]
    #[must_use]
    pub const fn damping(&self) -> f32 {
        self.damping
    }

    #[doc = include_str!("spring_track.doc/mass_2.md")]
    #[must_use]
    pub const fn mass(&self) -> f32 {
        self.mass
    }

    #[doc = include_str!("spring_track.doc/spring_progress.md")]
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
    #[doc = include_str!("spring_track.doc/sample.md")]
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
