//! The [`TimelinePosition`] snapshot of where a timeline sits (spec §4.17).

#[doc = include_str!("../../docs/api/timeline/timeline_position.doc/TimelinePosition.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimelinePosition {
    /// Logical seconds elapsed since the start of the *current* iteration.
    elapsed: f32,
    /// Normalized progress within the current iteration, in `0..=1` (raw and
    /// linear; easing is applied downstream, not here).
    progress: f32,
    /// The number of iterations already completed (0-based), used for repeats.
    iteration: u32,
}

impl TimelinePosition {
    /// The position at the very start of playback: zero elapsed, zero progress,
    /// no completed iterations.
    #[must_use]
    pub const fn start() -> Self {
        Self {
            elapsed: 0.0,
            progress: 0.0,
            iteration: 0,
        }
    }

    /// Creates a position from explicit `elapsed`, `progress` and `iteration`
    /// components. The caller is responsible for supplying consistent values;
    /// no normalization is performed.
    #[must_use]
    pub const fn new(elapsed: f32, progress: f32, iteration: u32) -> Self {
        Self {
            elapsed,
            progress,
            iteration,
        }
    }

    /// Returns the logical seconds elapsed within the current iteration.
    #[must_use]
    pub const fn elapsed(&self) -> f32 {
        self.elapsed
    }

    /// Returns the raw, linear normalized progress within the current iteration
    /// (`0..=1`).
    #[must_use]
    pub const fn progress(&self) -> f32 {
        self.progress
    }

    /// Returns the number of iterations already completed (0-based).
    #[must_use]
    pub const fn iteration(&self) -> u32 {
        self.iteration
    }

    /// Returns whether this is the position at the very start of playback:
    /// progress at (or below) zero and no iterations completed yet.
    #[must_use]
    pub const fn at_start(&self) -> bool {
        self.progress <= 0.0 && self.iteration == 0
    }

    /// Returns whether the current iteration has reached its end
    /// (`progress >= 1.0`).
    #[must_use]
    pub const fn is_complete_iteration(&self) -> bool {
        self.progress >= 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_is_all_zero_and_at_start() {
        let p = TimelinePosition::start();
        assert_eq!(p.elapsed(), 0.0);
        assert_eq!(p.progress(), 0.0);
        assert_eq!(p.iteration(), 0);
        assert!(p.at_start());
        assert!(!p.is_complete_iteration());
    }

    #[test]
    fn new_carries_components() {
        let p = TimelinePosition::new(0.5, 0.5, 2);
        assert_eq!(p.elapsed(), 0.5);
        assert_eq!(p.progress(), 0.5);
        assert_eq!(p.iteration(), 2);
    }

    #[test]
    fn at_start_requires_first_iteration() {
        // Progress zero but on a later iteration is not "at start".
        assert!(!TimelinePosition::new(0.0, 0.0, 1).at_start());
        assert!(TimelinePosition::new(0.0, 0.0, 0).at_start());
    }

    #[test]
    fn is_complete_iteration_at_full_progress() {
        assert!(TimelinePosition::new(1.0, 1.0, 0).is_complete_iteration());
        assert!(!TimelinePosition::new(0.99, 0.99, 0).is_complete_iteration());
    }
}
