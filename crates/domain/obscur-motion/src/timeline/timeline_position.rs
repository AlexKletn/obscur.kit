#![doc = include_str!("timeline_position.doc/_module.md")]

#[doc = include_str!("timeline_position.doc/TimelinePosition.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimelinePosition {
    #[doc = include_str!("timeline_position.doc/elapsed.md")]
    elapsed: f32,
    #[doc = include_str!("timeline_position.doc/progress.md")]
    progress: f32,
    #[doc = include_str!("timeline_position.doc/iteration.md")]
    iteration: u32,
}

impl TimelinePosition {
    #[doc = include_str!("timeline_position.doc/start.md")]
    #[must_use]
    pub const fn start() -> Self {
        Self {
            elapsed: 0.0,
            progress: 0.0,
            iteration: 0,
        }
    }

    #[doc = include_str!("timeline_position.doc/new.md")]
    #[must_use]
    pub const fn new(elapsed: f32, progress: f32, iteration: u32) -> Self {
        Self {
            elapsed,
            progress,
            iteration,
        }
    }

    #[doc = include_str!("timeline_position.doc/elapsed_2.md")]
    #[must_use]
    pub const fn elapsed(&self) -> f32 {
        self.elapsed
    }

    #[doc = include_str!("timeline_position.doc/progress_2.md")]
    #[must_use]
    pub const fn progress(&self) -> f32 {
        self.progress
    }

    #[doc = include_str!("timeline_position.doc/iteration_2.md")]
    #[must_use]
    pub const fn iteration(&self) -> u32 {
        self.iteration
    }

    #[doc = include_str!("timeline_position.doc/at_start.md")]
    #[must_use]
    pub const fn at_start(&self) -> bool {
        self.progress <= 0.0 && self.iteration == 0
    }

    #[doc = include_str!("timeline_position.doc/is_complete_iteration.md")]
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
