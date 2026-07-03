//! The [`TimelineState`] playback lifecycle (spec §4.17 "motion engine").

/// The lifecycle state of a [`Timeline`](super::Timeline) (spec §4.17).
///
/// A timeline moves `Idle` → `Running` when started, may toggle between
/// `Running` and `Paused` while it plays, and settles in `Finished` once all its
/// iterations are consumed. Because the engine is time-agnostic and
/// event-driven, these transitions are driven by explicit calls
/// (start/pause/resume/advance), never by a clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineState {
    /// Created but not yet started: holds its initial position and does not
    /// advance. This is the default state.
    #[default]
    Idle,
    /// Actively playing: [`advance`](super::Timeline::advance) integrates elapsed
    /// time and moves the position forward.
    Running,
    /// Started but temporarily suspended: retains its position and ignores
    /// `advance` until resumed.
    Paused,
    /// Completed: every iteration has been played out and the position rests at
    /// the end. A finished timeline does not advance further.
    Finished,
}

impl TimelineState {
    /// Returns whether the timeline is actively playing (`Running`).
    #[must_use]
    pub const fn is_running(self) -> bool {
        matches!(self, Self::Running)
    }

    /// Returns whether the timeline has completed (`Finished`).
    #[must_use]
    pub const fn is_finished(self) -> bool {
        matches!(self, Self::Finished)
    }

    /// Returns whether the timeline has started but not yet finished, i.e. it is
    /// `Running` or `Paused`.
    #[must_use]
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Running | Self::Paused)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_idle() {
        assert_eq!(TimelineState::default(), TimelineState::Idle);
    }

    #[test]
    fn is_running_only_for_running() {
        assert!(TimelineState::Running.is_running());
        assert!(!TimelineState::Idle.is_running());
        assert!(!TimelineState::Paused.is_running());
        assert!(!TimelineState::Finished.is_running());
    }

    #[test]
    fn is_finished_only_for_finished() {
        assert!(TimelineState::Finished.is_finished());
        assert!(!TimelineState::Running.is_finished());
    }

    #[test]
    fn is_active_for_running_and_paused() {
        assert!(TimelineState::Running.is_active());
        assert!(TimelineState::Paused.is_active());
        assert!(!TimelineState::Idle.is_active());
        assert!(!TimelineState::Finished.is_active());
    }
}
